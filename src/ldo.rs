//! Stack and Call structure of Lua

use std::rc::Rc;

use crate::{
    api::LuaError,
    luaG, luaU, luaY, luaZ,
    luaconf::LUAI_MAXRCALLS,
    object::{Closure, LClosure, ProtoRef, StkId, TValue, UpVal},
    state::{CallInfo, LuaState},
    LUA_MINSTACK, LUA_SIGNATURE,
};

/// type of protected functions, to be ran by `runprotected'
type Pfunc<T> = fn(&mut LuaState, T) -> Result<i32, LuaError>;

/// index in the CallInfo vector
pub type CallId = usize;

pub(crate) enum PrecallStatus {
    /// did a call to a Rust function
    Rust,
    /// initiated a call to a Lua function
    Lua,
    /// Rust function yielded
    RustYield,
}

pub struct SParser<T> {
    pub z: Option<luaZ::Zio<T>>,
    pub name: String,
}

impl<T> SParser<T> {
    pub fn new(z: luaZ::Zio<T>, name: &str) -> Self {
        Self {
            z: Some(z),
            name: name.to_owned(),
        }
    }
}

fn seterrorobj(state: &mut LuaState, errcode: &LuaError) {
    match errcode {
        LuaError::ErrorHandlerError => {
            state.push_string("error in error handling");
        }
        LuaError::SyntaxError | LuaError::RuntimeError => {
            let msg = state.stack.last().unwrap().clone();
            state.stack.push(msg);
        }
    }
}

impl LuaState {
    ///  Call a function (Rust or Lua). The function to be called is at stack[cl_stkid].
    ///  The arguments are on the stack, right after the function.
    ///  When returns, all the results are on the stack, starting at the original
    ///  function position.
    pub(crate) fn dcall(&mut self, cl_stkid: StkId, nresults: i32) -> Result<(), LuaError> {
        self.n_rcalls += 1;
        if self.n_rcalls >= LUAI_MAXRCALLS {
            if self.n_rcalls == LUAI_MAXRCALLS {
                return self.run_error("Rust stack overflow");
            } else if self.n_rcalls >= LUAI_MAXRCALLS + (LUAI_MAXRCALLS >> 3) {
                // error while handing stack error
                return Err(LuaError::ErrorHandlerError);
            }
        }
        if let TValue::Function(_cl) = &self.stack[cl_stkid] {
            if let PrecallStatus::Lua = self.dprecall(cl_stkid, nresults)? {
                self.vexecute(1)?;
            }
        }
        self.n_rcalls -= 1;
        Ok(())
    }

    pub(crate) fn dprecall(
        &mut self,
        cl_stkid: StkId,
        nresults: i32,
    ) -> Result<PrecallStatus, LuaError> {
        let cl_stkid = match &self.stack[cl_stkid] {
            TValue::Function(_) => cl_stkid,
            _ => {
                // func' is not a function. check the `function' metamethod
                // TODO
                //self.try_func_tag_method(cl_stkid)?
                luaG::type_error(self, cl_stkid, "call")?;
                unreachable!()
            }
        };
        let cl = if let TValue::Function(cl) = &self.stack[cl_stkid] {
            cl.clone()
        } else {
            unreachable!()
        };
        self.base_ci[self.ci].saved_pc = self.saved_pc;
        let cl=cl.borrow();
        match &*cl {
            Closure::Lua(cl) => {
                // Lua function. prepare its call
                let base = if self.protos[cl.proto].is_vararg {
                    // vararg function
                    let nargs = self.stack.len() - cl_stkid - 1;
                    self.adjust_varargs(cl.proto, nargs)
                } else {
                    // no varargs
                    let base = cl_stkid + 1;
                    if self.stack.len() > base + self.protos[cl.proto].numparams {
                        panic!("cannot truncate stack in dprecall");
                        //self.stack.truncate(base + cl.proto.numparams);
                    }
                    base
                };
                let ci = CallInfo {
                    func: cl_stkid,
                    base,
                    top: base + self.protos[cl.proto].maxstacksize,
                    nresults,
                    ..Default::default()
                };
                self.base = base;
                self.saved_pc = 0;
                self.stack.resize(ci.top, TValue::Nil);
                self.base_ci.push(ci);
                self.ci += 1;
                // TODO handle hooks
                Ok(PrecallStatus::Lua)
            }
            Closure::Rust(cl) => {
                // this is a Rust function, call it
                let ci = CallInfo {
                    func: cl_stkid,
                    base: cl_stkid + 1,
                    top: self.stack.len() + LUA_MINSTACK,
                    nresults,
                    ..Default::default()
                };
                self.base = cl_stkid + 1;
                self.base_ci.push(ci);
                self.ci += 1;
                // TODO handle hooks
                let n = (cl.f)(self).map_err(|_| LuaError::RuntimeError)?;
                if n < 0 {
                    Ok(PrecallStatus::RustYield)
                } else {
                    self.poscall(self.stack.len() as u32 - n as u32);
                    Ok(PrecallStatus::Rust)
                }
            }
        }
    }

    pub(crate) fn adjust_varargs(&mut self, proto: ProtoRef, nargs: usize) -> usize {
        let nfix_args = self.protos[proto].numparams;
        for _ in nargs..nfix_args {
            self.stack.push(TValue::Nil);
        }
        // move fixed parameters to final position
        let base = self.stack.len(); // final position of first argument
        let fixed_pos = base - nargs; // first fixed argument
        for i in 0..nfix_args {
            let value = self.stack.remove(fixed_pos + i);
            self.stack.insert(fixed_pos + i, TValue::Nil);
            self.stack.push(value);
        }
        base
    }
    pub(crate) fn _try_func_tag_method(&self, _cl_stkid: StkId) -> Result<StkId, LuaError> {
        todo!()
    }
}

pub fn pcall<T>(
    state: &mut LuaState,
    func: Pfunc<T>,
    u: T,
    old_top: StkId,
    ef: StkId,
) -> Result<i32, LuaError> {
    let old_errfunc;
    let old_allowhook;
    let old_ci;
    let old_n_ccalls;
    {
        old_n_ccalls = state.n_rcalls;
        old_ci = state.ci;
        old_allowhook = state.allowhook;
        old_errfunc = state.errfunc;
        state.errfunc = ef;
    }
    let status = func(state, u);
    if let Err(e) = &status {
        state.close_func(old_top);
        seterrorobj(state, e);
        state.n_rcalls = old_n_ccalls;
        state.ci = old_ci;
        state.base = state.base_ci[state.ci].base;
        state.saved_pc = state.base_ci[state.ci].saved_pc;
        state.allowhook = old_allowhook;
    }
    state.errfunc = old_errfunc;
    status
}

fn f_parser<T>(state: &mut LuaState, parser: &mut SParser<T>) -> Result<i32, LuaError> {
    let c = if let Some(ref mut z) = parser.z {
        z.look_ahead(state)
    } else {
        unreachable!()
    };
    let proto = if c == LUA_SIGNATURE.chars().next() {
        luaU::undump
    } else {
        luaY::parser
    }(state, parser)?;
    let nups = proto.nups;
    let protoid = state.protos.len();
    state.protos.push(proto);
    let mut luacl = LClosure::new(protoid, Rc::clone(&state.l_gt));
    for _ in 0..nups {
        luacl.upvalues.push(UpVal::default());
    }
    let cl = Closure::Lua(luacl);
    state.stack.push(TValue::from(cl));
    Ok(0)
}

pub fn protected_parser<T>(
    state: &mut LuaState,
    zio: luaZ::Zio<T>,
    chunk_name: &str,
) -> Result<i32, LuaError> {
    let mut p = SParser::new(zio, chunk_name);
    let top = state.stack.len();
    let errfunc = state.errfunc;
    pcall(state, f_parser, &mut p, top, errfunc)
}
