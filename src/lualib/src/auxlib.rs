//! Auxiliary functions for building Lua libraries

use crate::{
    api::{self, LuaError},
    luaH::TableRef,
    object::TValue,
    state::LuaState,
    LuaFloat, LuaInteger, LuaRustFunction, LUA_MULTRET, LUA_REGISTRYINDEX,
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

pub fn error(state: &mut LuaState, msg: &str) -> Result<(), LuaError> {
    lwhere(state, 1);
    state.push_string(msg);
    api::concat(state, 2)?;
    api::error(state)
}

fn lwhere(_state: &mut LuaState, _arg: i32) {
    todo!();
}

pub fn typename(s: &LuaState, index: isize) -> String {
    s.index2adr(index).get_type_name().to_owned()
}

pub fn check_number(s: &mut LuaState, index: isize) -> Result<LuaFloat, ()> {
    match api::to_number(s, index) {
        None => {
            type_error(s, index, "number").map_err(|_| ())?;
            unreachable!()
        }
        Some(value) => Ok(value),
    }
}

pub fn check_numeral(s: &mut LuaState, index: isize) -> Result<LuaFloat, ()> {
    match s.index2adr(index) {
        TValue::Float(n) => Ok(n),
        TValue::Integer(i) => Ok(i as LuaFloat),
        _ => {
            type_error(s, index, "number").map_err(|_| ())?;
            unreachable!()
        }
    }
}

pub fn check_boolean(s: &mut LuaState, index: isize) -> Result<bool, ()> {
    let value = api::to_boolean(s, index);
    Ok(value)
}

pub fn check_integer(s: &mut LuaState, index: isize) -> Result<LuaInteger, ()> {
    match api::to_integer(s, index) {
        None => {
            arg_error(s, index, "number has no integer representation").map_err(|_| ())?;
            unreachable!()
        }
        Some(value) => Ok(value),
    }
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
            let _ = type_error(s, index, "table");
            Err(())
        }
    }
}

pub(crate) fn type_error(
    s: &mut LuaState,
    index: isize,
    expected_type: &str,
) -> Result<(), LuaError> {
    let value = s.index2adr(index);
    let tname = value.get_type_name();
    let msg = format!("{} expected, got {}", expected_type, tname);
    s.push_string(&msg);
    arg_error(s, index, &msg)
}

pub(crate) fn arg_error(
    state: &mut LuaState,
    narg: isize,
    extra_msg: &str,
) -> Result<(), LuaError> {
    // TODO
    state.push_string(&format!("bad argument #{} ({})", narg, extra_msg));
    Err(LuaError::RuntimeError)
}

pub fn opt_integer(state: &mut LuaState, narg: i32) -> Option<LuaInteger> {
    check_integer(state, narg as isize).ok()
}

pub fn opt_number(state: &mut LuaState, narg: i32) -> Option<LuaFloat> {
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

/// set functions from list 'l' into table at top - 'nup'; each
/// function gets the 'nup' elements at the top as upvalues.
/// Returns with only the table at the stack.
pub fn set_funcs(state: &mut LuaState, funcs: &[LibReg], nup: i32) {
    for f in funcs {
        for _ in 0..nup {
            // copy upvalues to the top
            api::push_value(state, -nup as isize);
        }
        api::push_rust_function(state, f.func, nup as usize); // closure with those upvalues
        api::set_field(state, -(nup + 2) as isize, f.name);
    }
    state.pop_stack(nup as usize); // remove upvalues
}

pub fn get_meta_field(s: &mut LuaState, obj: i32, event: &str) -> bool {
    if !api::get_meta_table(s, obj) {
        // no metatable
        false
    } else {
        api::push_string(s, event);
        api::raw_get(s, -2);
        if api::is_nil(s, -1) {
            api::pop(s, 2); // remove metatable and metafield
            false
        } else {
            api::remove(s, -2); // remove only metatable
            true
        }
    }
}

/// Creates a new table and registers there the functions in list `funcs`
pub fn new_lib(state: &mut LuaState, funcs: &[LibReg]) {
    api::create_table(state);
    set_funcs(state, funcs, 0);
}

///  If the registry already has the key tname, returns false.
/// Otherwise, creates a new table to be used as a metatable for userdata,
/// adds it to the registry with key tname, and returns true.
/// In both cases pushes onto the stack the final value associated with tname
/// in the registry.
pub(crate) fn new_metatable(s: &mut LuaState, tname: &str) -> bool {
    get_meta_table(s, tname); // try to get metatable
    if !api::is_nil(s, -1) {
        // name already in use?
        false // leave previous value on top, but return false
    } else {
        api::pop(s, 1);
        api::new_table(s); // create metatable
        api::push_value(s, -1);
        api::set_field(s, LUA_REGISTRYINDEX, tname); // registry.name = metatable
        true
    }
}

/// Pushes onto the stack the metatable associated with name tname in the registry
fn get_meta_table(s: &mut LuaState, tname: &str) {
    api::get_field(s, LUA_REGISTRYINDEX, tname);
}

/// Calls function openf with string modname as an argument
/// and sets the call result in package.loaded[modname],
/// as if that function has been called through require.
/// If glb is true, also stores the result into global modname.
/// Leaves a copy of that result on the stack.
pub fn requiref(
    s: &mut LuaState,
    modname: &str,
    openf: LuaRustFunction,
    glb: bool,
) -> Result<(), LuaError> {
    api::push_rust_function(s, openf, 0);
    api::push_string(s, modname); // argument to open function
    api::call(s, 1, 1)?; // open module
    get_sub_table(s, LUA_REGISTRYINDEX, "_LOADED");
    api::push_value(s, -2); // make copy of module (call result)
    api::set_field(s, -2, modname); // _LOADED[modname] = module
    api::pop(s, 1); // remove _LOADED table
    if glb {
        api::push_value(s, -1); // copy of 'mod'
        api::set_global(s, modname); // _G[modname] = module
    }
    Ok(())
}

/// Ensures that the value t[fname], where t is the value at index idx,
/// is a table, and pushes that table onto the stack.
/// Returns true if it finds a previous table there
/// and false if it creates a new table.
pub fn get_sub_table(s: &mut LuaState, idx: isize, fname: &str) -> bool {
    api::get_field(s, idx, fname);
    if api::is_table(s, -1) {
        true
    } else {
        api::pop(s, 1);
        let idx = api::abs_index(s, idx);
        api::new_table(s);
        api::push_value(s, -1);
        api::set_field(s, idx, fname);
        false
    }
}

pub fn check_any(s: &mut LuaState, index: isize) -> Result<(), LuaError> {
    if s.is_index_valid(index) {
        Ok(())
    } else {
        arg_error(s, index, "value expected")
    }
}
