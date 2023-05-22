use crate::{LuaState, state::PanicFunction};

#[derive(Debug)]
pub enum LuaError {
    RuntimeError,
    MemoryAllocationError,
    ErrorHandlerError,
}

pub fn newstate() -> Result<LuaState,()> {
    Ok(LuaState::default())
}

pub fn at_panic(state: &mut LuaState, panic:PanicFunction) -> Option<PanicFunction> {
    let old=state.g.panic.take();
    state.g.panic = Some(panic);
    old
}

pub fn tolstring(_state: &LuaState, _idx: i32, _len: Option<&mut usize> ) -> String {
    // TODO
    String::new()
}

pub fn pcall(_state:&mut LuaState, _nargs:usize, _nresults: Option<usize>, _errfunc: i32) -> Result<(),LuaError> {
    // TODO
    Ok(())
}