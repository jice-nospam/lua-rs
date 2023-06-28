//! Auxiliary functions for building Lua libraries

use crate::{
    api::{self, LuaError},
    state::LuaState,
    LuaInteger, LuaNumber, LUA_GLOBALSINDEX, LUA_MULTRET, LUA_REGISTRYINDEX, object::TValue, luaH::TableRef,
};

pub use crate::libs::*;

fn panic(state: &mut LuaState) -> i32 {
    eprintln!(
        "PANIC: unprotected error in call to Lua API ({})",
        api::to_string(state, -1).unwrap()
    );
    0
}
fn get_s(_state: &mut LuaState, ud: &&str, buff: &mut Vec<char>) -> Result<(), ()> {
    if ud.is_empty() {
        return Err(());
    }
    if !buff.is_empty() {
        return Err(());
    }
    let mut to_append = ud.chars().collect();
    buff.append(&mut to_append);
    Ok(())
}

pub fn newstate() -> LuaState {
    let mut state = crate::state::newstate();
    api::at_panic(&mut state, panic);
    state
}

pub fn loadstring(state: &mut LuaState, s: &str) -> Result<i32, LuaError> {
    api::load(state, get_s, s, Some(s))
}

pub fn dostring(state: &mut LuaState, s: &str) -> Result<i32, LuaError> {
    loadstring(state, s).and_then(|_| api::pcall(state, 0, LUA_MULTRET, 0))
}

pub(crate) fn register(
    state: &mut LuaState,
    lib_name: &str,
    funcs: &[LibReg],
) -> Result<(), LuaError> {
    open_lib(state, Some(lib_name), funcs, 0)
}

fn open_lib(
    state: &mut LuaState,
    lib_name: Option<&str>,
    funcs: &[LibReg],
    nupvalues: isize,
) -> Result<(), LuaError> {
    if let Some(lib_name)=  lib_name {
        // check whether lib already exists
        find_table(state, LUA_REGISTRYINDEX, "_LOADED");
        state.get_field(-1, lib_name); // get _LOADED[libname]
        if !state.is_table(-1) {
            // not found?
            state.pop_stack(1); // remove previous result
                                // try global variable (and create one if it does not exist)
            if find_table(state, LUA_GLOBALSINDEX, lib_name).is_some() {
                return error(state,&format!("name conflict for module '{}'", lib_name));
            }
            state.push_value(-1);
            state.set_field(-3, lib_name); // _LOADED[libname] = new table
        }
        state.remove(-2); // remove _LOADED table
        state.insert(-(nupvalues + 1)); // move library table to below upvalues
    }
    for lib_reg in funcs.iter() {
        for _ in 0..nupvalues {
            state.push_value(-nupvalues); // copy upvalues to the top
        }
        state.push_rust_closure(lib_reg.func, nupvalues as usize);
        state.set_field(-(nupvalues + 2), lib_reg.name);
        //println!("register {}.{}",lib_name.unwrap_or(""),lib_reg.name);
    }
    state.pop_stack(nupvalues as usize); // remove upvalues
    Ok(())
}

pub fn error(state: &mut LuaState, msg: &str) -> Result<(), LuaError> {
    lwhere(state,1);
    state.push_string(msg);
    api::concat(state,2)?;
    api::error(state)
}

fn lwhere(_state: &mut LuaState, _arg: i32) {
    todo!();
}

pub(crate) fn find_table(state: &mut LuaState, index: isize, name: &str) -> Option<String> {
    state.push_value(index);
    for module in name.split('.') {
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

pub fn typename(s: &LuaState, index: isize) -> String {
    s.index2adr(index).get_type_name().to_owned()
}

pub fn check_number(s: &mut LuaState, index: isize) -> Result<LuaNumber, ()> {
    let value = api::to_number(s, index);
    if value == 0.0 && !api::is_number(s, index) {
        type_error(s, index, "number").map_err(|_| ())?;
    }
    Ok(value)
}

pub fn check_boolean(s: &mut LuaState, index: isize) -> Result<bool, ()> {
    let value = api::to_boolean(s, index);
    Ok(value)
}

pub fn check_integer(s: &mut LuaState, index: isize) -> Result<LuaInteger, ()> {
    let value = api::to_number(s, index);
    if value == 0.0 && !api::is_number(s, index) {
        type_error(s, index, "number").map_err(|_| ())?;
    }
    Ok(value as LuaInteger)
}

pub fn check_string(s: &mut LuaState, index: isize) -> Result<String, ()> {
    match api::to_string(s, index) {
        Some(s) => Ok(s),
        None => {
            type_error(s, index, "string").map_err(|_| ())?;
            unreachable!()
        }
    }
}

pub fn check_table(s: &mut LuaState, index: isize) -> Result<TableRef, ()> {
    match s.index2adr(index) {
        TValue::Table(tref) => Ok(tref.clone()),
        _ => {
            let _= type_error(s, index, "table");
            Err(())
        },
    }
}

pub(crate) fn type_error(s: &mut LuaState, index: isize, expected_type: &str) -> Result<(), LuaError> {
    let value = s.index2adr(index);
    let tname = value.get_type_name();
    let msg=format!("{} expected, got {}", expected_type, tname);
    s.push_string(&msg);
    arg_error(s, index, &msg)
}

pub(crate) fn arg_error(state: &mut LuaState, narg: isize, extra_msg: &str) -> Result<(), LuaError> {
    // TODO
    state.push_string(&format!("bad argument #{} ({})", narg, extra_msg));
    Err(LuaError::RuntimeError)
}

pub fn opt_int(state: &mut LuaState, narg: i32) -> Option<i32> {
    check_integer(state, narg as isize).ok().map(|n| n as i32)
}

pub fn opt_number(state: &mut LuaState, narg: i32) -> Option<LuaNumber> {
    check_number(state, narg as isize).ok()
}

pub fn opt_boolean(state: &mut LuaState, narg: i32) -> Option<bool> {
    check_boolean(state, narg as isize).ok()
}

pub fn opt_table(state: &mut LuaState, narg: i32) -> Option<TableRef> {
    check_table(state, narg as isize).ok()
}

pub fn opt_string(state: &mut LuaState, narg: i32) -> Option<String> {
    check_string(state, narg as isize).ok()
}

pub fn obj_len(state: &mut LuaState, idx: i32) -> usize {
    match state.index2adr(idx as isize) {
        TValue::String(s) => s.len(),
        TValue::UserData(_udref) => todo!(),
        TValue::Table(tref) => tref.borrow().len(),
        _ => 0,
    }
}
