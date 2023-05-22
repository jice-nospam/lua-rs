//! Stack and Call structure of Lua

use std::rc::Rc;

use crate::{
    api::LuaError,
    luaF, luaU, luaY, luaZ,
    luaconf::LUAI_MAXRCALLS,
    object::{Closure, LClosure, StkId, TValue},
    state::{CallInfo, LuaState},
    LuaStateRef, LUA_MINSTACK, LUA_SIGNATURE,
};

/// type of protected functions, to be ran by `runprotected'
type Pfunc<T> = fn(LuaStateRef, T) -> Result<i32, LuaError>;

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

fn seterrorobj(state: LuaStateRef, errcode: &LuaError) {
    match errcode {
        LuaError::ErrorHandlerError => {
            let mut state = state.borrow_mut();
            state.push_string("error in error handling");
        }
        LuaError::SyntaxError | LuaError::RuntimeError => {
            let mut state = state.borrow_mut();
            let msg = state.stack.last().unwrap().clone();
            state.stack.push(msg);
        }
    }
}

pub fn rawrunprotected<T>(
    stateref: LuaStateRef,
    func: Pfunc<T>,
    user_data: T,
) -> Result<i32, LuaError> {
    func(stateref, user_data)
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
                // func' is not a function. check the `function' tag method
                self.try_func_tag_method(cl_stkid)?
            }
        };
        let cl = if let TValue::Function(cl) = &self.stack[cl_stkid] {
            cl.clone()
        } else {
            unreachable!()
        };
        self.base_ci[self.ci].saved_pc = self.saved_pc;
        match cl.as_ref() {
            Closure::Lua(cl) => {
                // Lua function. prepare its call
                let base = if cl.proto.is_vararg {
                    // vararg function
                    let nargs = self.stack.len() - cl_stkid - 1;
                    self.adjust_varargs(&cl.proto, nargs)
                } else {
                    // no varargs
                    let base = cl_stkid + 1;
                    if self.stack.len() > base + cl.proto.numparams {
                        panic!("cannot truncate stack in dprecall");
                        //self.stack.truncate(base + cl.proto.numparams);
                    }
                    base
                };
                let mut ci = CallInfo::default();
                ci.func = cl_stkid;
                ci.base = base;
                self.base = base;
                ci.top = base + cl.proto.maxstacksize;
                self.saved_pc = 0;
                ci.nresults = nresults;
                self.stack.resize(ci.top, TValue::Nil);
                self.base_ci.push(ci);
                self.ci+=1;
                // TODO handle hooks
                return Ok(PrecallStatus::Lua);
            }
            Closure::Rust(cl) => {
                // this is a Rust function, call it
                let mut ci = CallInfo::default();
                ci.func = cl_stkid;
                self.base = cl_stkid + 1;
                ci.base = cl_stkid + 1;
                ci.nresults = nresults;
                ci.top = self.stack.len() + LUA_MINSTACK;
                self.base_ci.push(ci);
                self.ci+=1;
                // TODO handle hooks
                let n = (cl.f)(self);
                if n < 0 {
                    return Ok(PrecallStatus::RustYield);
                } else {
                    return Ok(PrecallStatus::Rust);
                }
            }
        }
    }

    pub(crate) fn adjust_varargs(&mut self, proto: &crate::object::Proto, nargs: usize) -> usize {
        let nfix_args=proto.numparams;
        for _ in nargs..nfix_args {
            self.stack.push(TValue::Nil);
        }
        // move fixed parameters to final position
        let base = self.stack.len(); // final position of first argument
        let fixed_pos = base - nargs; // first fixed argument
        for i in 0..nfix_args {
            let value = self.stack.remove(fixed_pos+i);
            self.stack.insert(fixed_pos+i, TValue::Nil);
            self.stack.push(value);
        }
        base
    }
    pub(crate) fn try_func_tag_method(&self, _cl_stkid: StkId) -> Result<StkId, LuaError> {
        todo!()
    }
}

pub fn pcall<T>(
    stateref: LuaStateRef,
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
        let mut state = stateref.borrow_mut();
        old_n_ccalls = state.n_rcalls;
        old_ci = state.ci;
        old_allowhook = state.allowhook;
        old_errfunc = state.errfunc;
        state.errfunc = ef;
    }
    let status = rawrunprotected(Rc::clone(&stateref), func, u);
    if let Err(e) = &status {
        luaF::close(Rc::clone(&stateref), old_top);
        seterrorobj(Rc::clone(&stateref), e);
        let mut state = stateref.borrow_mut();
        state.n_rcalls = old_n_ccalls;
        state.ci = old_ci;
        state.base = state.base_ci[state.ci].base;
        state.saved_pc = state.base_ci[state.ci].saved_pc;
        state.allowhook = old_allowhook;
    }
    stateref.borrow_mut().errfunc = old_errfunc;
    status
}

fn f_parser<T>(state: LuaStateRef, parser: &mut SParser<T>) -> Result<i32, LuaError> {
    let c = if let Some(ref mut z) = parser.z {
        z.look_ahead()
    } else {
        unreachable!()
    };
    let proto = if c == LUA_SIGNATURE.chars().next() {
        luaU::undump
    } else {
        luaY::parser
    }(Rc::clone(&state), parser)?;
    let mut state = state.borrow_mut();
    let cl = Closure::Lua(LClosure::new(proto, Rc::clone(&state.l_gt)));
    state.stack.push(TValue::Function(Rc::new(cl)));
    Ok(0)
}

pub fn protected_parser<T>(
    state: LuaStateRef,
    zio: luaZ::Zio<T>,
    chunk_name: &str,
) -> Result<i32, LuaError> {
    let mut p = SParser::new(zio, chunk_name);
    let top = state.borrow().stack.len();
    let errfunc = state.borrow().errfunc;
    pcall(state, f_parser, &mut p, top, errfunc)
}
