//! Lua API

use std::rc::Rc;

use crate::{
    luaD, luaG, luaV, luaZ,
    object::{Closure, TValue},
    state::{LuaState, PanicFunction},
    LuaInteger, LuaNumber, LuaRustFunction, Reader, LUA_REGISTRYINDEX, LUA_RIDX_GLOBALS,
};

#[derive(Debug, PartialEq)]
pub enum LuaError {
    /// error during error handling
    ErrorHandlerError,
    /// error during a function execution
    RuntimeError,
    /// error during parsing of the source code
    SyntaxError,
}

pub fn at_panic(state: &mut LuaState, panic: PanicFunction) -> Option<PanicFunction> {
    let old = state.g.panic.take();
    state.g.panic = Some(panic);
    old
}

pub fn to_string(state: &mut LuaState, idx: isize) -> Option<String> {
    // TODO convert in stack
    match state.index2adr(idx) {
        TValue::String(s) => Some(s.as_ref().clone()),
        TValue::Number(n) => Some(format!("{}", n)),
        _ => None,
    }
}

struct CallData {
    func: u32,
    nresults: i32,
}

fn f_call(state: &mut LuaState, c: &CallData) -> Result<i32, LuaError> {
    state.dcall(c.func as usize, c.nresults, false)?;
    Ok(0)
}

pub fn pcall(
    state: &mut LuaState,
    nargs: usize,
    nresults: i32,
    errfunc: u32,
) -> Result<i32, LuaError> {
    let (c, func) = {
        debug_assert!(state.stack.len() > nargs);
        state.check_results(nargs, nresults);
        let c = CallData {
            func: (state.stack.len() - (nargs + 1)) as u32,
            nresults,
        };
        (c, errfunc)
    };
    let status = luaD::pcall(state, f_call, &c, c.func as usize, func as usize)?;
    state.adjust_results(nresults);
    Ok(status)
}

pub fn load<T>(
    state: &mut LuaState,
    reader: Reader<T>,
    data: T,
    name: Option<&str>,
) -> Result<i32, LuaError> {
    let zio = luaZ::Zio::new(reader, data);
    let res = luaD::protected_parser(state, zio, name.unwrap_or("?"));
    if res.is_ok() {
        if let TValue::Function(clref) = state.stack.last().unwrap() {
            if let Closure::Lua(lcl) = &mut *clref.borrow_mut() {
                if lcl.upvalues.len() == 1 {
                    // does it have one upvalue?
                    let gt = state.get_global_table();
                    lcl.upvalues[0].value = gt.clone();
                }
            }
        }
    }
    Ok(0)
}

/// Returns the index of the top element in the stack.
/// Because indices start at 1, this result is equal to the number of
/// elements in the stack (and so 0 means an empty stack).
pub fn get_top(s: &mut LuaState) -> usize {
    s.stack.len() - s.base_ci[s.ci].base
}

/// Pushes onto the stack the value of the global name.
pub fn get_global(s: &mut LuaState, name: &str) {
    let gt = s.get_global_table();
    let top = s.stack.len();
    s.stack.push(TValue::from(name));
    LuaState::get_tablev2(&mut s.stack, &gt, &TValue::from(name), Some(top));
}

/// Pops a value from the stack and sets it as the new value of global name.
pub fn set_global(state: &mut LuaState, name: &str) {
    let gt = state.get_global_table();
    let key = TValue::from(name);
    let value = state.stack.pop().unwrap();
    LuaState::set_tablev(state, &gt, key, value);
}

/// Pushes onto the stack the value t[k], where t is the value at the given index.
pub fn get_field(s: &mut LuaState, index: isize, name: &str) {
    let t = s.index2adr(index).clone();
    let key = TValue::from(name);
    LuaState::get_tablev2(&mut s.stack, &t, &key, None);
}

/// Pushes a copy of the element at the given index onto the stack.
pub fn push_value(s: &mut LuaState, index: isize) {
    s.push_value(index);
}

/// Pushes the string `value` onto the stack.
pub fn push_literal(s: &mut LuaState, value: &str) {
    s.push_literal(value);
}

pub fn call(s: &mut LuaState, nargs: usize, nresults: i32) -> Result<(), LuaError> {
    s.call(nargs, nresults, 0, None)
}

pub fn set_field(s: &mut LuaState, idx: isize, name: &str) {
    let key = TValue::from(name);
    let value = s.stack.pop().unwrap();
    let idx = if idx < 0 && idx > LUA_REGISTRYINDEX {
        idx + 1
    } else {
        idx
    };
    let t = s.index2adr(idx as isize);
    s.set_tablev(&t, key, value);
}

pub fn pop(s: &mut LuaState, count: usize) {
    s.pop_stack(count);
}

pub fn push_string(s: &mut LuaState, value: &str) {
    s.push_string(value);
}

pub fn push_number(s: &mut LuaState, value: LuaNumber) {
    s.push_number(value);
}

pub fn push_boolean(s: &mut LuaState, value: bool) {
    s.push_boolean(value);
}

pub fn push_nil(s: &mut LuaState) {
    s.push_nil();
}

pub fn to_number(s: &mut LuaState, index: isize) -> LuaNumber {
    // TODO convert in stack
    s.index2adr(index).get_number_value()
}

pub fn to_integer(s: &mut LuaState, index: isize) -> LuaInteger {
    // TODO convert in stack
    s.index2adr(index).get_number_value() as LuaInteger
}

pub fn to_boolean(s: &mut LuaState, index: isize) -> bool {
    // TODO convert in stack
    s.index2adr(index).is_false()
}

pub fn is_number(s: &mut LuaState, index: isize) -> bool {
    s.index2adr(index).is_number()
}

pub fn is_boolean(s: &mut LuaState, index: isize) -> bool {
    s.index2adr(index).is_boolean()
}

pub fn is_string(s: &mut LuaState, index: isize) -> bool {
    s.index2adr(index).is_string()
}

pub fn is_nil(s: &mut LuaState, index: isize) -> bool {
    s.index2adr(index).is_nil()
}

pub fn is_function(s: &mut LuaState, index: isize) -> bool {
    s.index2adr(index).is_function()
}

/// Returns true if the value at the given index is a table, and false otherwise.
pub fn is_table(s: &mut LuaState, index: isize) -> bool {
    s.index2adr(index).is_table()
}

pub fn to_pointer(s: &mut LuaState, index: isize) -> *const TValue {
    let index = if index < 0 {
        s.stack.len() - (-index) as usize
    } else {
        index as usize + s.base_ci[s.ci].base
    };
    &s.stack[index as usize] as *const TValue
}

pub fn concat(state: &mut LuaState, n: usize) -> Result<(), LuaError> {
    if n >= 2 {
        luaV::concat(state, n)?;
    } else if n == 0 {
        // push empty string
        state.push_string("");
    } // else n == 1, nothing to do
    Ok(())
}

pub(crate) fn error(state: &mut LuaState) -> Result<(), LuaError> {
    luaG::error_msg(state)
}

pub fn create_table(state: &mut LuaState) {
    state.create_table();
}

pub(crate) fn _replace(_state: &mut LuaState, _lua_environindex: isize) {
    todo!()
}

pub(crate) fn set_metatable(state: &mut LuaState, obj_index: i32) {
    let mt = state.stack.pop().unwrap();
    let mt = if mt.is_nil() {
        None
    } else {
        if let TValue::Table(tref) = mt {
            Some(tref)
        } else {
            unreachable!()
        }
    };
    let obj_index = if obj_index < 0 && obj_index > LUA_REGISTRYINDEX as i32 {
        obj_index + 1
    } else {
        obj_index
    };
    let obj = state.index2adr(obj_index as isize);
    match obj {
        TValue::Table(tref) => {
            tref.borrow_mut().metatable = mt;
        }
        TValue::UserData(udref) => {
            udref.borrow_mut().metatable = mt;
        }
        _ => {
            let obj_type = obj.get_type_name().to_owned();
            state.g.mt.insert(obj_type, mt);
        }
    }
}

pub(crate) fn raw_get_i(state: &mut LuaState, idx: i32, n: i32) {
    let o = state.index2adr(idx as isize);
    if let TValue::Table(tref) = o {
        let value = {
            let mut t = tref.borrow_mut();
            t.get_num(n as usize).clone()
        };
        state.stack.push(value);
    } else {
        unreachable!()
    }
}

pub fn push_rust_function(state: &mut LuaState, func: LuaRustFunction, nupval: usize) {
    if nupval == 0 {
        state.push_rust_function(func);
    } else {
        todo!()
    }
}

pub fn set_top(s: &mut LuaState, idx: i32) {
    if idx >= 0 {
        while s.stack.len() < s.base_ci[s.ci].base + idx as usize {
            s.push_nil();
        }
    } else {
        let newlen = s.stack.len() + 1 - (-idx) as usize;
        s.stack.resize(newlen, TValue::Nil);
    }
}

pub fn next(s: &mut LuaState, idx: i32) -> bool {
    let t = s.index2adr(idx as isize);
    if let TValue::Table(tref) = t {
        let t = tref.borrow();
        let (k, v) = t.next(s.stack.last().unwrap());
        s.stack.pop(); // remove old key
        if k.is_nil() {
            // no more elements.
            return false;
        }
        s.stack.push(k); // new key
        s.stack.push(v);
        true
    } else {
        unreachable!()
    }
}

pub fn push_global_table(state: &mut LuaState) {
    raw_get_i(state, LUA_REGISTRYINDEX as i32, LUA_RIDX_GLOBALS as i32);
}

pub fn raw_get(s: &mut LuaState, idx: i32) {
    let t = s.index2adr(idx as isize);
    debug_assert!(t.is_table());
    if let TValue::Table(tref) = &t {
        let mut t = tref.borrow_mut();
        let key = s.stack.pop().unwrap();
        let value = t.get(&key).cloned().unwrap_or(TValue::Nil);
        // replace key with result
        let len = s.stack.len();
        s.stack[len - 1] = value;
    }
}

pub fn get_meta_table(s: &mut LuaState, objindex: i32) -> bool {
    let obj = s.index2adr(objindex as isize);
    let mt = match obj {
        TValue::Table(tref) => tref.borrow().metatable.clone(),
        TValue::UserData(_) => {
            todo!()
        }
        _ => {
            // get global type metatable
            let objtype = obj.get_type_name();
            s.g.mt.get(objtype).cloned().flatten()
        }
    };
    match mt {
        None => false,
        Some(tref) => {
            s.stack.push(TValue::Table(Rc::clone(&tref)));
            true
        }
    }
}

/// Removes the element at the given valid index,
/// shifting down the elements above this index to fill the gap.
/// This function cannot be called with a pseudo-index,
/// because a pseudo-index is not an actual stack position.
pub fn remove(s: &mut LuaState, idx: isize) {
    debug_assert!(idx < s.stack.len() as isize);
    debug_assert!(idx >= -(s.stack.len() as isize));
    // convert to absolute index
    let idx = if idx < 0 {
        s.stack.len() as isize + idx
    } else {
        idx
    };
    s.stack.remove(idx as usize);
}

/// Creates a new empty table and pushes it onto the stack.
pub fn new_table(s: &mut LuaState) {
    create_table(s);
}

/// Converts the acceptable index idx into an absolute index
/// (that is, one that does not depend on the stack top).
pub fn abs_index(s: &mut LuaState, idx: isize) -> isize {
    if idx > 0 || idx <= LUA_REGISTRYINDEX {
        idx
    } else {
        s.stack.len() as isize + idx - s.base_ci[s.ci].func as isize
    }
}

pub fn is_none_or_nil(s: &mut LuaState, index: i32) -> bool {
    !s.is_index_valid(index as isize) || s.index2adr(index as isize).is_nil()
}
