//! Stack and Call structure of Lua

use crate::{
    api::LuaError,
    luaG, luaU, luaY, luaZ,
    luaconf::LUAI_MAXRCALLS,
    object::{Closure, StkId, TValue},
    state::{CallInfo, LuaState, CIST_FRESH, CIST_RUST, CIST_TAIL},
    LUA_MINSTACK, LUA_MULTRET, LUA_SIGNATURE,
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
    pub(crate) fn dcall_no_yield(
        &mut self,
        cl_stkid: StkId,
        nresults: i32,
    ) -> Result<(), LuaError> {
        self.nny += 1;
        self.dcall(cl_stkid, nresults)?;
        self.nny -= 1;
        Ok(())
    }
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
        if self.stack[cl_stkid].is_function() {
            if let PrecallStatus::Lua = self.dprecall(cl_stkid, nresults)? {
                // is a Lua function ?
                self.base_ci[self.ci].call_status |= CIST_FRESH; // mark that it is a "fresh" execute
                self.vexecute()?; // call it
            }
        }
        self.n_rcalls -= 1;
        Ok(())
    }

    /// Prepares the call to a function (C or Lua). For C functions, also do
    /// the call. The function to be called is at 'func'.  The arguments
    /// are on the stack, right after the function.  Creates the CallInfo
    /// to be executed, if it was a Lua function. Otherwise (a Rust function)
    /// returns with all the results on the stack, starting at the
    /// original function position.
    pub(crate) fn dprecall(
        &mut self,
        func: StkId,
        nresults: i32,
    ) -> Result<PrecallStatus, LuaError> {
        let func = match &self.stack[func] {
            TValue::Function(_) => func,
            _ => {
                // func' is not a function. check the `function' metamethod
                // TODO
                //self.try_func_tag_method(cl_stkid)?
                luaG::type_error(self, func, "call")?;
                unreachable!()
            }
        };
        let cl = self.get_closure_ref(func);
        let cl = cl.borrow();
        match &*cl {
            Closure::Lua(cl) => {
                // Lua function. prepare its call
                let nargs = self.stack.len() - func - 1; // number of real arguments
                let n_fix_params = self.protos[cl.proto].numparams;
                let frame_size = self.protos[cl.proto].maxstacksize;
                let ci = CallInfo {
                    func,
                    top: func + 1 + frame_size,
                    nresults,
                    ..Default::default()
                };
                // complete missing arguments
                for _ in nargs..n_fix_params {
                    self.stack.push(TValue::Nil);
                }
                self.base_ci.push(ci);
                self.ci += 1;
                Ok(PrecallStatus::Lua)
            }
            Closure::Rust(cl) => {
                // this is a Rust function, call it
                self.precall_rust(func, nresults, cl)?;
                Ok(PrecallStatus::Rust)
            }
        }
    }

    /// precall for rust functions
    fn precall_rust(
        &mut self,
        func: usize,
        nresults: i32,
        cl: &crate::object::RClosure,
    ) -> Result<i32, LuaError> {
        let ci = CallInfo {
            func,
            top: self.stack.len() + LUA_MINSTACK,
            nresults,
            call_status: CIST_RUST,
            ..Default::default()
        };
        self.base_ci.push(ci);
        self.ci += 1;
        // TODO handle hooks
        let n = (cl.f)(self).map_err(|_| LuaError::RuntimeError)?;
        // do the actual call

        self.poscall(n)?;
        Ok(n)
    }

    /// Prepare a function for a tail call, building its call info on top
    /// of the current call info. 'narg1' is the number of arguments plus 1
    /// (so that it includes the function itself). Return the number of
    /// results, if it was a C function, or -1 for a Lua function.
    pub(crate) fn dpre_tailcall(
        &mut self,
        func: usize,
        narg1: usize,
        delta: usize,
        nresults: &mut i32,
    ) -> Result<crate::luaD::PrecallStatus, LuaError> {
        let func = match &self.stack[func] {
            TValue::Function(_) => func,
            _ => {
                // func' is not a function. check the `function' metamethod
                // TODO
                //self.try_func_tag_method(cl_stkid)?
                luaG::type_error(self, func, "call")?;
                unreachable!()
            }
        };
        let cl = self.get_closure_ref(func);
        let cl = cl.borrow();
        match &*cl {
            Closure::Lua(cl) => {
                let frame_size = self.protos[cl.proto].maxstacksize;
                let n_fix_params = self.protos[cl.proto].numparams;
                self.base_ci[self.ci].func -= delta; // restore func (if vararg)
                let new_func = self.base_ci[self.ci].func;
                for i in 0..narg1 {
                    // move down function and arguments
                    self.set_stack_from_idx(new_func + 1, func + i);
                }
                let new_func = self.base_ci[self.ci].func;
                for i in narg1..n_fix_params {
                    // complete missing arguments
                    self.set_stack_from_value(new_func + i, TValue::Nil);
                }
                self.base_ci[self.ci].top = new_func + 1 + frame_size;
                self.base_ci[self.ci].saved_pc = 0; // starting point
                self.base_ci[self.ci].call_status |= CIST_TAIL;
                self.stack.resize(new_func + narg1, TValue::Nil);
                Ok(PrecallStatus::Lua)
            }
            Closure::Rust(cl) => {
                *nresults = self.precall_rust(func, LUA_MULTRET, cl)?;
                Ok(PrecallStatus::Rust)
            }
        }
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
    let old_nny;
    {
        old_ci = state.ci;
        old_allowhook = state.allowhook;
        old_errfunc = state.errfunc;
        old_nny = state.nny;
        state.errfunc = ef;
    }
    let status = func(state, u);
    if let Err(e) = &status {
        state.close_func(old_top);
        seterrorobj(state, e);
        state.ci = old_ci;
        state.allowhook = old_allowhook;
        state.nny = old_nny;
    }
    state.errfunc = old_errfunc;
    status
}

fn f_parser<T>(state: &mut LuaState, parser: &mut SParser<T>) -> Result<i32, LuaError> {
    let c = if let Some(ref mut z) = parser.z {
        z.look_ahead(state) // read first character
    } else {
        unreachable!()
    };
    let cl = if c == LUA_SIGNATURE.chars().next() {
        luaU::undump
    } else {
        luaY::parser
    }(state, parser)?;
    let cl = Closure::Lua(cl);
    state.stack.push(TValue::from(cl));
    Ok(0)
}

pub fn protected_parser<T>(
    state: &mut LuaState,
    zio: luaZ::Zio<T>,
    chunk_name: &str,
) -> Result<i32, LuaError> {
    state.nny += 1; // cannot yield during parsing
    let mut p = SParser::new(zio, chunk_name);
    let top = state.stack.len();
    let errfunc = state.errfunc;
    let status = pcall(state, f_parser, &mut p, top, errfunc);
    state.nny -= 1;
    status
}
