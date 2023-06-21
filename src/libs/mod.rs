//! Initialization of libraries for lua
mod base;
mod io;
mod maths;
mod string;
mod table;
use crate::{api::LuaError, state::LuaState, LuaRustFunction};

use self::{base::lib_open_base, io::lib_open_io, maths::lib_open_math, string::lib_open_string, table::lib_open_table};

pub struct LibReg<'a> {
    pub name: &'a str,
    pub func: LuaRustFunction,
}

const LUA_LIBS: [LibReg; 5] = [
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
];

pub fn open_libs(state: &mut LuaState) -> Result<(), LuaError> {
    for reg in LUA_LIBS.iter() {
        state.push_rust_function(reg.func);
        state.push_string(reg.name);
        state.call(1, 0)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{luaL, object::TValue};
    #[test]
    fn open_libs_defines_g() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();

        let mut l_gt = state.l_gt.borrow_mut();
        let _g = l_gt.get(&TValue::from("_G"));
        assert!(_g.is_some());
    }
    #[test]
    fn open_libs_load_baselib() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();

        let mut l_gt = state.l_gt.borrow_mut();
        let print = l_gt.get(&TValue::from("print"));
        assert!(print.is_some());
        assert!(if let Some(TValue::Function(_)) = print {
            true
        } else {
            false
        });
    }
}
