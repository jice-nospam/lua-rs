//! Basic library

use crate::{api, lex::str2d, luaL, state::LuaState, LuaRustFunction, LUA_VERSION};

use super::LibReg;

const BASE_FUNCS: [LibReg; 21] = [
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
        name: "getmetatable",
        func: luab_getmetatable,
    },
    LibReg {
        name: "ipairs",
        func: luab_ipairs,
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
        name: "next",
        func: luab_next,
    },
    LibReg {
        name: "pairs",
        func: luab_pairs,
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
        name: "rawlen",
        func: luab_rawlen,
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
        name: "xpcall",
        func: luab_xpcall,
    },
];
pub fn luab_assert(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_dofile(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_error(_state: &mut LuaState) -> Result<i32, ()> {
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
                    _ = write!(s.stdout, "\t");
                }
                _ = write!(s.stdout, "{}", svalue);
                api::pop(s, 1);
            }
            _ => {
                luaL::error(s, "'tostring' must return a string to 'print'").map_err(|_| ())?;
                unreachable!()
            }
        }
    }
    _ = writeln!(s.stdout);
    Ok(0)
}
pub fn luab_rawequal(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn luab_rawlen(_state: &mut LuaState) -> Result<i32, ()> {
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
        let n = luaL::check_number(state, 1)?;
        api::push_number(state, n);
        return Ok(1);
    }
    if !(2..=36).contains(&base) {
        luaL::arg_error(state, 2, "base out of range").map_err(|_| ())?;
    }
    let s1 = luaL::check_string(state, 1).map_err(|_| ())?;
    if let Some(n) = str2d(&s1) {
        api::push_number(state, n);
        return Ok(1);
    }
    // not a number
    api::push_nil(state);
    Ok(1)
}
pub fn luab_tostring(s: &mut LuaState) -> Result<i32, ()> {
    // TODO hangle metamethods
    let value = s.index2adr(1);
    let svalue = format!("{}", value);
    api::push_string(s, &svalue);
    Ok(1)
}
pub fn luab_type(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}

pub fn luab_xpcall(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}

fn pairs_meta(
    s: &mut LuaState,
    method: &str,
    is_zero: bool,
    iter: LuaRustFunction,
) -> Result<i32, ()> {
    if !luaL::get_meta_field(s, 1, method) {
        // no metamethod?
        luaL::check_table(s, 1)?; // argument must be a table
        api::push_rust_function(s, iter, 0); // will return generator,
        api::push_value(s, 1); // state,
        if is_zero {
            api::push_number(s, 0.0); // and initial value
        } else {
            api::push_nil(s);
        }
    } else {
        api::push_value(s, 1); // argument 'self' to metamethod
        api::call(s, 1, 3).map_err(|_| ())?; // get 3 values from metamethod
    }
    Ok(3)
}

/// 'ipairs' function. Returns 'ipairsaux', given "table", 0.
/// (The given "table" may not be a table.)
pub fn luab_ipairs(s: &mut LuaState) -> Result<i32, ()> {
    luaL::check_any(s, 1).map_err(|_| ())?;
    api::push_rust_function(s, ipairs_aux, 0); // iteration function
    api::push_value(s, 1); // state
    api::push_integer(s, 0); // initial value
    Ok(3)
}
pub fn ipairs_aux(s: &mut LuaState) -> Result<i32, ()> {
    let i = luaL::check_integer(s, 2)? + 1; // next value
    luaL::check_table(s, 1)?;
    api::push_integer(s, i);
    api::raw_get_i(s, 1, i as usize);
    if api::is_nil(s, -1) {
        Ok(1)
    } else {
        Ok(2)
    }
}
pub fn luab_pairs(s: &mut LuaState) -> Result<i32, ()> {
    pairs_meta(s, "__pairs", false, luab_next)
}

pub fn luab_next(s: &mut LuaState) -> Result<i32, ()> {
    luaL::check_table(s, 1)?;
    api::set_top(s, 2); // create a 2nd argument if there isn't one
    if api::next(s, 1) {
        Ok(2)
    } else {
        api::push_nil(s);
        Ok(1)
    }
}

pub fn lib_open_base(state: &mut LuaState) -> Result<i32, ()> {
    // set global _G
    api::push_global_table(state);
    api::push_global_table(state);
    api::set_field(state, -2, "_G");
    // open lib into global table
    luaL::set_funcs(state, &BASE_FUNCS, 0);
    // set global _VERSION
    api::push_literal(state, LUA_VERSION);
    api::set_field(state, -2, "_VERSION");
    Ok(1)
}

#[cfg(test)]
mod tests {
    use crate::{api, luaL, object::TValue, LUA_VERSION};
    #[test]
    fn baselib_defines_g() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        api::get_global(&mut state, "_G");
        assert!(matches!(state.stack.last().unwrap(), TValue::Table(_)));
    }
    #[test]
    fn baselib_defines_print() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();

        api::get_global(&mut state, "print");
        assert!(matches!(state.stack.last().unwrap(), TValue::Function(_)));
    }
    #[test]
    fn baselib_defines_version() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        api::get_global(&mut state, "_VERSION");
        assert!(*state.stack.last().unwrap() == TValue::from(LUA_VERSION));
    }
    #[test]
    fn print_vararg() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(&mut state, "print('hello',' ','world')").unwrap();
    }
    #[test]
    fn global_env() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(&mut state, "a=3 z=_G.a").unwrap();
        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(3));
    }
    #[test]
    fn ipairs() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(
            &mut state,
            "t={1,3,6}
            z=0
            for _,v in ipairs(t) do
                z = z + v
            end",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(10));
    }
    #[test]
    fn pairs_array() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(
            &mut state,
            "t={1,3,6}
            z=0
            for k,v in pairs(t) do
                z = z + v
            end",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(10));
    }
    #[test]
    fn pairs_hash() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(
            &mut state,
            "t={a=1,b=3,c=6}
            z=0
            for k,v in pairs(t) do
                z = z + v
            end",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(10));
    }
}
