//! Auxiliary functions for building Lua libraries

use std::rc::Rc;

use crate::{
    api::{self, LuaError},
     LuaStateRef, LUA_GLOBALSINDEX, LUA_REGISTRYINDEX, LUA_MULTRET, state::LuaState,
};

pub use crate::libs::*;

fn panic(state: LuaStateRef) -> i32 {
    eprintln!(
        "PANIC: unprotected error in call to Lua API ({})",
        api::to_string(&mut state.borrow_mut(), -1)
    );
    0
}
fn get_s(_state: LuaStateRef, ud: &&str, buff: &mut Vec<char>) -> Result<(), ()> {
    if ud.is_empty() {
        return Err(());
    }
    if ! buff.is_empty() {
        return Err(());
    }
    let mut to_append = ud.chars().collect();
    buff.append(&mut to_append);
    Ok(())
}

pub fn newstate() -> LuaStateRef {
    let state = crate::state::newstate();
    api::at_panic(Rc::clone(&state), panic);
    state
}

pub fn loadbuffer(state: LuaStateRef, s: &str, name: Option<&str>) -> Result<i32, LuaError> {
    api::load(state, get_s, s, name)
}

pub fn loadstring(state: LuaStateRef, s: &str) -> Result<i32, LuaError> {
    loadbuffer(state, s, Some(s))
}

pub fn dostring(state: LuaStateRef, s: &str) -> Result<i32, LuaError> {
    loadstring(Rc::clone(&state), s).and_then(|_| api::pcall(state, 0, LUA_MULTRET, 0))
}

pub(crate) fn register(
    state: &mut crate::state::LuaState,
    lib_name: &str,
    funcs: &[LibReg],
) -> Result<(), LuaError> {
    open_lib(state, Some(lib_name), funcs, 0)
}

fn open_lib(
    state: &mut crate::state::LuaState,
    lib_name: Option<&str>,
    funcs: &[LibReg],
    nupvalues: isize,
) -> Result<(), LuaError> {
    match lib_name {
        Some(lib_name) => {
            // check whether lib already exists
            find_table(state, LUA_REGISTRYINDEX, "_LOADED");
            state.get_field(-1, lib_name); // get _LOADED[libname]
            if !state.is_table(-1) {
                // not found?
                state.pop_stack(1); // remove previous result
                                    // try global variable (and create one if it does not exist)
                if find_table(state, LUA_GLOBALSINDEX, lib_name).is_some() {
                    return error(format!("name conflict for module '{}'", lib_name));
                }
                state.push_value(-1);
                state.set_field(-3, lib_name); // _LOADED[libname] = new table
            }
            state.remove(-2);
            state.insert(-(nupvalues + 1));
        }
        None => (),
    }
    for lib_reg in funcs.iter() {
        for _ in 0..nupvalues {
            state.push_value(-nupvalues);
        }
        state.push_rust_closure(lib_reg.func, nupvalues as usize);
        state.set_field(-(nupvalues + 2), lib_reg.name);
    }
    state.pop_stack(nupvalues as usize);
    Ok(())
}

fn error(_msg: String) -> Result<(), LuaError> {
    todo!()
}

fn find_table(state: &mut crate::state::LuaState, index: isize, name: &str) -> Option<String> {
    state.push_value(index);
    for module in name.split(".") {
        state.push_string(module);
        state.rawget(-2);
        if state.is_nil(-1) {
            state.pop_stack(1);
            state.create_table();
            state.push_string(module);
            state.push_value(-2);
            state.set_table(-4);
        } else if !state.is_table(-1) {
            state.pop_stack(2);
            return Some(module.to_owned());
        }
        state.remove(-2);
    }
    None
}

pub fn typename(s: &LuaState, index: isize) -> &str {
    api::typename(s, api::get_type(s,index))
}