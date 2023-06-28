//! Basic library

use crate::{
    api, api::LuaError, luaL, object::TValue, state::LuaState, LuaNumber, LuaRustFunction,
    LUA_GLOBALSINDEX, LUA_VERSION,
};

use super::LibReg;

const BASE_FUNCS: [LibReg; 22] = [
    LibReg {
        name: "assert",
        func: luab_assert,
    },
    LibReg {
        name: "dofile",
        func: luab_dofile,
    },
    LibReg {
        name: "error",
        func: luab_error,
    },
    LibReg {
        name: "getfenv",
        func: luab_getfenv,
    },
    LibReg {
        name: "getmetatable",
        func: luab_getmetatable,
    },
    LibReg {
        name: "loadfile",
        func: luab_loadfile,
    },
    LibReg {
        name: "load",
        func: luab_load,
    },
    LibReg {
        name: "loadstring",
        func: luab_loadstring,
    },
    LibReg {
        name: "next",
        func: luab_next,
    },
    LibReg {
        name: "pcall",
        func: luab_pcall,
    },
    LibReg {
        name: "print",
        func: luab_print,
    },
    LibReg {
        name: "rawequal",
        func: luab_rawequal,
    },
    LibReg {
        name: "rawget",
        func: luab_rawget,
    },
    LibReg {
        name: "rawset",
        func: luab_rawset,
    },
    LibReg {
        name: "select",
        func: luab_select,
    },
    LibReg {
        name: "setfenv",
        func: luab_setfenv,
    },
    LibReg {
        name: "setmetatable",
        func: luab_setmetatable,
    },
    LibReg {
        name: "tonumber",
        func: luab_tonumber,
    },
    LibReg {
        name: "tostring",
        func: luab_tostring,
    },
    LibReg {
        name: "type",
        func: luab_type,
    },
    LibReg {
        name: "unpack",
        func: luab_unpack,
    },
    LibReg {
        name: "xpcall",
        func: luab_xpcall,
    },
];

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

pub fn luab_cocreate(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_coresume(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_corunning(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_costatus(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_cowrap(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_yield(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}

pub fn luab_assert(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_dofile(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_error(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_getfenv(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_getmetatable(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_load(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_loadfile(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_loadstring(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_pcall(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
/// If your system does not support `stdout', you can just remove this function.
/// If you need, you can define your own `print' function, following this
/// model but changing `println!' to put the strings at a proper place
/// (a console window or a log file, for instance).
pub fn luab_print(s: &mut LuaState) -> Result<i32, ()> {
    let n = api::get_top(s) as isize; // number of arguments
    api::get_global(s, "tostring");
    for i in 1..=n {
        api::push_value(s, -1); // function to be called
        api::push_value(s, i); // value to print
        api::call(s, 1, 1).map_err(|_| ())?;
        match api::to_string(s, -1) {
            // get result
            Some(svalue) => {
                if i > 1 {
                    print!("\t");
                }
                print!("{}", svalue);
                api::pop(s, 1);
            }
            _ => {
                luaL::error(s, "'tostring' must return a string to 'print'").map_err(|_| ())?;
                unreachable!()
            }
        }
    }
    println!();
    Ok(0)
}
pub fn luab_rawequal(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_rawget(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_rawset(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_select(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_setfenv(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_setmetatable(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_tonumber(state: &mut LuaState) -> Result<i32, ()> {
    let base = if api::get_top(state) == 2 {
        10
    } else {
        luaL::check_integer(state, 2).map_err(|_| ())?
    };
    if base == 10 {
        // standard conversion
        let n = api::to_number(state, 1);
        api::push_number(state, n);
        return Ok(1);
    }
    if !(2..=36).contains(&base) {
        luaL::arg_error(state, 2, "base out of range").map_err(|_| ())?;
    }
    let s1 = luaL::check_string(state, 1).map_err(|_| ())?;
    if let Ok(n) = s1.parse::<LuaNumber>() {
        api::push_number(state, n);
        return Ok(1);
    }
    // not a number
    api::push_nil(state);
    Ok(1)
}
pub fn luab_tostring(s: &mut LuaState) -> Result<i32, ()> {
    // TODO hangle metamethods
    match s.index2adr(1) {
        TValue::Number(_) => {
            let value = api::to_string(s, 1).unwrap();
            api::push_string(s, &value);
        }
        TValue::String(_) => api::push_value(s, 1),
        TValue::Boolean(_) => {
            let value = api::to_boolean(s, 1);
            api::push_string(s, if value { "true" } else { "false" });
        }
        TValue::Nil => api::push_string(s, "nil"),
        _ => {
            let ptr = api::to_pointer(s, 1);
            let value = format!("{} : {:?}", luaL::typename(s, 1), ptr);
            api::push_string(s, &value);
        }
    }
    Ok(1)
}
pub fn luab_type(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_unpack(state: &mut LuaState) -> Result<i32, ()> {
    luaL::check_table(state, 1).map_err(|_| ())?;
    let mut i=luaL::opt_int(state,2).unwrap_or(1);
    let len=luaL::obj_len(state,1);
    let e = luaL::opt_int(state,3).unwrap_or(len as i32);
    let n=e-i+1; // number of elements
    if n <= 0 {
        return Ok(0); // empty range
    }
    while i <= e {
        api::raw_get_i(state,1,i);
        i+=1;
    }
    Ok(n)
}
pub fn luab_xpcall(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}

pub fn luab_ipairs(s: &mut LuaState) -> Result<i32, ()> {
    luaL::check_table(s, 1)?;
    api::push_value(s, LUA_GLOBALSINDEX-1); // generator
    api::push_value(s, 1); // state
    api::push_number(s, 0.0); // and initial value
    Ok(3)
}
pub fn ipairsaux(s: &mut LuaState) -> Result<i32, ()> {
    let i =luaL::check_integer(s, 2)? + 1; // next value
    luaL::check_table(s, 1)?;
    api::push_number(s, i as f64);
    api::raw_get_i(s, 1, i as i32);
    if api::is_nil(s, -1) {
        Ok(0)
    } else {
        Ok(2)
    }
}
pub fn luab_pairs(s: &mut LuaState) -> Result<i32, ()> {
    luaL::check_table(s, 1)?;
    api::push_value(s, LUA_GLOBALSINDEX-1); // generator
    api::push_value(s, 1); // state
    api::push_nil(s); // and initial value
    Ok(3)
}
pub fn luab_next(s: &mut LuaState) -> Result<i32, ()> {
    luaL::check_table(s, 1)?;
    api::set_top(s,2); // create a 2nd argument if there isn't one
    if api::next(s,1) {
        Ok(2)
    } else {
        api::push_nil(s);
        Ok(1)
    }
}
pub fn luab_newproxy(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}

pub fn lib_open_base(state: &mut LuaState) -> Result<i32, ()> {
    base_open(state).unwrap();
    luaL::register(state, "coroutine", &CO_FUNCS).unwrap();
    Ok(2)
}

fn base_open(state: &mut LuaState) -> Result<(), LuaError> {
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
    state.push_value(-1); // `w' will be its own metatable
    state.set_metatable(-2);
    state.push_literal("kv");
    state.set_field(-2, "__mode"); // metatable(w).__mode = "kv"
    state.push_rust_closure(luab_newproxy, 1);
    state.set_global("newproxy");
    Ok(())
}

fn auxopen(state: &mut LuaState, name: &str, f: LuaRustFunction, u: LuaRustFunction) {
    state.push_rust_function(u);
    state.push_rust_closure(f, 1);
    state.set_field(-2, name);
}

#[cfg(test)]
mod tests {
    use crate::{luaL, object::TValue, api};
    #[test]
    fn print_vararg() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(&mut state, "print('hello',' ','world')").unwrap();
    }
    #[test]
    fn unpack() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(&mut state, "a,b=unpack({3,5})").unwrap();
        api::get_global(&mut state, "a");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(3.0));
        api::get_global(&mut state, "b");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(5.0));
    }
    #[test]
    fn global_env() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(&mut state, "a=3 z=_G.a").unwrap();
        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(3.0));
    }
}
