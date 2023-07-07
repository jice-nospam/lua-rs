//! Coroutine Library

use crate::{luaL, state::LuaState, LuaError};

use super::LibReg;

const CO_FUNCS: [LibReg; 6] = [
    LibReg {
        name: "create",
        func: luab_cocreate,
    },
    LibReg {
        name: "resume",
        func: luab_coresume,
    },
    LibReg {
        name: "running",
        func: luab_corunning,
    },
    LibReg {
        name: "status",
        func: luab_costatus,
    },
    LibReg {
        name: "wrap",
        func: luab_cowrap,
    },
    LibReg {
        name: "yield",
        func: luab_yield,
    },
];

pub fn luab_cocreate(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn luab_coresume(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn luab_corunning(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn luab_costatus(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn luab_cowrap(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn luab_yield(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}

pub fn lib_open_coro(state: &mut LuaState) -> Result<i32, LuaError> {
    luaL::new_lib(state, &CO_FUNCS);
    Ok(1)
}
