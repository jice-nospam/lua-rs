//! Library for Table Manipulation

use crate::{luaL, state::LuaState, api};

use super::LibReg;

const TAB_FUNCS: [LibReg; 6] = [
    LibReg {
        name: "concat",
        func: tconcat,
    },
    LibReg {
        name: "insert",
        func: tinsert,
    },
    LibReg {
        name: "pack",
        func: tpack,
    },
    LibReg {
        name: "unpack",
        func: tunpack,
    },
    LibReg {
        name: "remove",
        func: tremove,
    },
    LibReg {
        name: "sort",
        func: sort,
    },
];

pub fn tunpack(state: &mut LuaState) -> Result<i32, ()> {
    luaL::check_table(state, 1).map_err(|_| ())?;
    let mut i = luaL::opt_int(state, 2).unwrap_or(1);
    let len = luaL::obj_len(state, 1);
    let e = luaL::opt_int(state, 3).unwrap_or(len as i32);
    if i > e {
        return Ok(0); // empty range
    }
    let n = e - i + 1; // number of elements
    if n <= 0 {
        return Ok(0); // empty range
    }
    api::raw_get_i(state, 1, i);
    while i < e {
        i += 1;
        api::raw_get_i(state, 1, i);
    }
    Ok(n)
}

pub fn tconcat(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn tpack(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn tinsert(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn tremove(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn sort(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}

pub fn lib_open_table(state: &mut LuaState) -> Result<i32, ()> {
    luaL::new_lib(state, &TAB_FUNCS);
    Ok(1)
}

#[cfg(test)]
mod tests {
    use crate::{api, luaL, object::TValue};
    #[test]
    fn unpack() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(&mut state, "a,b=table.unpack({3,5})").unwrap();
        api::get_global(&mut state, "a");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(3.0));
        api::get_global(&mut state, "b");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(5.0));
    }
}
