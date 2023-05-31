//! Initialization of libraries for lua
mod base;
mod string;
mod maths;
mod io;
use crate::{LuaRustFunction,  api::LuaError, state::LuaState};

use self::{base::lib_open_base, string::lib_open_string, maths::lib_open_math, io::lib_open_io};

pub struct LibReg<'a> {
    pub name: &'a str,
    pub func : LuaRustFunction,
}

const LUA_LIBS:[LibReg;4] =[
    LibReg {name:"", func: lib_open_base},
    LibReg {name:"string", func: lib_open_string},
    LibReg {name:"math", func: lib_open_math},
    LibReg {name:"io", func: lib_open_io},
];

pub fn open_libs(state:&mut LuaState) -> Result<(), LuaError> {
    for reg in LUA_LIBS.iter() {
        state.push_rust_function(reg.func);
        state.push_string(reg.name);
        state.call(1,0)?;
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
        let _g = l_gt.get(&TValue::new_string("_G"));
        assert!(_g.is_some());
    }
    #[test]
    fn open_libs_load_baselib() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();

        let mut l_gt = state.l_gt.borrow_mut();
        let print = l_gt.get(&TValue::new_string("print"));
        assert!(print.is_some());
        assert!(if let Some(TValue::Function(_)) = print { true } else {false});
    }
}
