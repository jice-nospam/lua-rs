//! Initialization of libraries for lua
mod base;
mod io;
mod maths;
mod string;
mod table;
mod coro;
use crate::{luaL,api,api::LuaError, state::LuaState, LuaRustFunction, LUA_REGISTRYINDEX};

use self::{
    base::lib_open_base, io::lib_open_io, maths::lib_open_math, string::lib_open_string,
    table::lib_open_table,coro::lib_open_coro
};

pub struct LibReg<'a> {
    pub name: &'a str,
    pub func: LuaRustFunction,
}

const LUA_LIBS: [LibReg; 6] = [
    LibReg {
        name: "",
        func: lib_open_base,
    },
    LibReg {
        name: "string",
        func: lib_open_string,
    },
    LibReg {
        name: "math",
        func: lib_open_math,
    },
    LibReg {
        name: "io",
        func: lib_open_io,
    },
    LibReg {
        name: "table",
        func: lib_open_table,
    },
    LibReg {
        name: "coroutine",
        func: lib_open_coro,
    },
    // TODO os and debug ?
];

// Opens all standard Lua libraries into the given state. 
pub fn open_libs(state: &mut LuaState) -> Result<(), LuaError> {
    // call open functions from 'loadedlibs' and set results to global table
    for reg in LUA_LIBS.iter() {
        luaL::requiref(state, reg.name, reg.func, true)?;
        api::pop(state,1); // remove lib 
    }
    // add open functions from 'preloadedlibs' into 'package.preload' table
    luaL::get_sub_table(state, LUA_REGISTRYINDEX, "_PRELOAD");
    for reg in LUA_LIBS.iter() {
        api::push_rust_function(state, reg.func, 0);
        api::set_field(state, -2, reg.name);
    }
    api::pop(state,1); // remove _PRELOAD table
    Ok(())
}
