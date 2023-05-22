//! Initialization of libraries for lua
mod base;
use crate::{LuaRustFunction, state::LuaStateRef, api::LuaError};

use self::base::lib_open_base;

pub struct LibReg<'a> {
    pub name: &'a str,
    pub func : LuaRustFunction,
}

const LUA_LIBS:[LibReg;1] =[
    LibReg {name:"", func: lib_open_base}
];

pub fn open_libs(state:LuaStateRef) -> Result<(), LuaError> {
    let mut state=state.borrow_mut();
    for reg in LUA_LIBS.iter() {
        state.push_rust_function(reg.func);
        state.push_string(reg.name);
        state.call(1,0)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{luaL, object::TValue};
    #[test]
    fn open_libs_defines_g() {
        let state = luaL::newstate();
        luaL::open_libs(Rc::clone(&state)).unwrap();

        let state=state.borrow_mut();
        let mut l_gt = state.l_gt.borrow_mut();
        let _g = l_gt.get(&TValue::new_string("_G"));
        assert!(_g.is_some());
    }
    #[test]
    fn open_libs_load_baselib() {
        let state = luaL::newstate();
        luaL::open_libs(Rc::clone(&state)).unwrap();

        let state=state.borrow_mut();
        let mut l_gt = state.l_gt.borrow_mut();
        let print = l_gt.get(&TValue::new_string("print"));
        assert!(print.is_some());
        assert!(if let Some(TValue::Function(_)) = print { true } else {false});
    }
}
