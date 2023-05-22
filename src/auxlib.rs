use crate::{LuaState, tostring, api::{self, LuaError}};

fn panic(state: &LuaState) -> i32 {
    eprintln!("PANIC: unprotected error in call to Lua API ({})", tostring(state,-1));
    return 0;
}

pub fn newstate() -> Result<LuaState,()> {
    let mut res=api::newstate();
    if let Ok(ref mut state) = res {
        api::at_panic(state, panic);
    }
    res
}

pub fn loadstring(_state: &mut LuaState, _s: &str) -> Result<(),LuaError> {
    // TODO
    Ok(())
}

pub fn dostring(state: &mut LuaState, s: &str) -> Result<(),LuaError> {
    loadstring(state, s).and_then(|()| {api::pcall(state,0,None,0)})
}