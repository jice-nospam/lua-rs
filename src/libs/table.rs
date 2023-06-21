//! Library for Table Manipulation

use crate::{luaL, state::LuaState};

use super::LibReg;

const TAB_FUNCS: [LibReg; 9] = [
    LibReg {
        name: "concat",
        func: tconcat,
    },
    LibReg {
        name: "foreach",
        func: foreach,
    },
    LibReg {
        name: "foreachi",
        func: foreachi,
    },
    LibReg {
        name: "getn",
        func: getn,
    },
    LibReg {
        name: "maxn",
        func: maxn,
    },
    LibReg {
        name: "insert",
        func: tinsert,
    },
    LibReg {
        name: "remove",
        func: tremove,
    },
    LibReg {
        name: "setn",
        func: setn,
    },
    LibReg {
        name: "sort",
        func: sort,
    },
];

pub fn lib_open_table(state: &mut LuaState) -> Result<i32, ()> {
    luaL::register(state, "table", &TAB_FUNCS).unwrap();
    Ok(1)
}

pub fn tconcat(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn foreach(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn foreachi(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn getn(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn maxn(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn tinsert(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn tremove(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn setn(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn sort(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
