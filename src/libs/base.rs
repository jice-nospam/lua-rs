//! Basic library

use crate::{luaL,api,state::LuaState, LUA_GLOBALSINDEX, LUA_VERSION, LuaRustFunction, api::LuaError, LuaType};

use super::LibReg;

const BASE_FUNCS:[LibReg;22] =[
    LibReg {name:"assert", func: luab_assert},
    LibReg {name:"dofile", func: luab_dofile},
    LibReg {name:"error", func: luab_error},
    LibReg {name:"getfenv", func: luab_getfenv},
    LibReg {name:"getmetatable", func: luab_getmetatable},
    LibReg {name:"loadfile", func: luab_loadfile},
    LibReg {name:"load", func: luab_load},
    LibReg {name:"loadstring", func: luab_loadstring},
    LibReg {name:"next", func: luab_next},
    LibReg {name:"pcall", func: luab_pcall},
    LibReg {name:"print", func: luab_print},
    LibReg {name:"rawequal", func: luab_rawequal},
    LibReg {name:"rawget", func: luab_rawget},
    LibReg {name:"rawset", func: luab_rawset},
    LibReg {name:"select", func: luab_select},
    LibReg {name:"setfenv", func: luab_setfenv},
    LibReg {name:"setmetatable", func: luab_setmetatable},
    LibReg {name:"tonumber", func: luab_tonumber},
    LibReg {name:"tostring", func: luab_tostring},
    LibReg {name:"type", func: luab_type},
    LibReg {name:"unpack", func: luab_unpack},
    LibReg {name:"xpcall", func: luab_xpcall}
];

const CO_FUNCS:[LibReg;6] =[
    LibReg {name:"create", func: luab_cocreate},
    LibReg {name:"resume", func: luab_coresume},
    LibReg {name:"running", func: luab_corunning},
    LibReg {name:"status", func: luab_costatus},
    LibReg {name:"wrap", func: luab_cowrap},
    LibReg {name:"yield", func: luab_yield},
];

pub fn luab_cocreate(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_coresume(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_corunning(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_costatus(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_cowrap(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_yield(_state: &mut LuaState) -> i32 {
    0
}

pub fn luab_assert(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_dofile(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_error(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_getfenv(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_getmetatable(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_load(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_loadfile(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_loadstring(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_next(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_pcall(_state: &mut LuaState) -> i32 {
    0
}
/// If your system does not support `stdout', you can just remove this function.
/// If you need, you can define your own `print' function, following this
/// model but changing `println!' to put the strings at a proper place
/// (a console window or a log file, for instance).
pub fn luab_print(s: &mut LuaState) -> i32 {
    let n = api::get_top(s) as isize; // number of arguments
    api::get_global(s,"tostring");
    for i in 1..=n {
        api::push_value(s,-1); // function to be called
        api::push_value(s,i); // value to print
        api::call(s,1,1);
        let svalue = api::to_string(s, -1); // get result
        if i > 1 {
            print!("\t");
        }
        print!("{}",svalue);
        api::pop(s,1);
    }
    println!();
    0
}
pub fn luab_rawequal(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_rawget(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_rawset(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_select(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_setfenv(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_setmetatable(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_tonumber(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_tostring(s: &mut LuaState) -> i32 {
    // TODO hangle metamethods
    match api::get_type(s,1) {
        LuaType::Number => {
            let value=api::to_string(s,1);
            api::push_string(s, &value);
        },
        LuaType::String => api::push_value(s, 1),
        LuaType::Boolean => {
            let value = api::to_boolean(s,1);
            api::push_string(s, if value {"true"} else {"false"});
        },
        LuaType::Nil => api::push_string(s,"nil"),
        _ => {
            let ptr=api::to_pointer(s,1);
            let value=format!("{} : {:?}", luaL::typename(s,1),ptr);
            api::push_string(s, &value);
        }
    }
    0
}
pub fn luab_type(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_unpack(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_xpcall(_state: &mut LuaState) -> i32 {
    0
}

pub fn luab_ipairs(_state: &mut LuaState) -> i32 {
    0
}
pub fn ipairsaux(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_pairs(_state: &mut LuaState) -> i32 {
    0
}
pub fn luab_newproxy(_state: &mut LuaState) -> i32 {
    0
}

pub fn lib_open_base(state: &mut LuaState) -> i32 {
    base_open(state).unwrap();
    luaL::register(state,"coroutine", &CO_FUNCS).unwrap();
    2
}

fn base_open(state: &mut LuaState) -> Result<(),LuaError> {
    // set global _G
    state.push_value(LUA_GLOBALSINDEX);
    state.set_global("_G");
    // open lib into global table
    luaL::register(state, "_G", &BASE_FUNCS)?;
    // set global _VERSION
    state.push_literal(LUA_VERSION);
    state.set_global("_VERSION");
    // `ipairs' and `pairs' need auxiliary functions as upvalues
    auxopen(state, "ipairs", luab_ipairs, ipairsaux);
    auxopen(state, "pairs", luab_pairs, luab_next);
    // `newproxy' needs a weaktable as upvalue
    state.create_table(); // new table `w'
    state.push_value(-1);// `w' will be its own metatable
    state.set_metatable(-2);
    state.push_literal("kv");
    state.set_field(-2, "__mode"); // metatable(w).__mode = "kv"
    state.push_rust_closure(luab_newproxy, 1);
    state.set_global("newproxy");
    Ok(())
}

fn auxopen(state: &mut LuaState, name: &str, f: LuaRustFunction, u: LuaRustFunction)  {
    state.push_rust_function(u);
    state.push_rust_closure(f, 1);
    state.set_field(-2, name);
}

