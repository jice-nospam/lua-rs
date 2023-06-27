//! Lua API

use crate::{
    luaD, luaG, luaV, luaZ,
    object::TValue,
    state::{LuaState, PanicFunction},
    Reader, LUA_GLOBALSINDEX, LuaNumber, LUA_REGISTRYINDEX, LuaInteger, LuaRustFunction,
};

#[derive(Debug,PartialEq)]
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
        TValue::Number(n) => {
            Some(format!("{}", n))
        },
        _ => None,
    }
}

struct CallData {
    func: u32,
    nresults: i32,
}

fn f_call(state: &mut LuaState, c: &CallData) -> Result<i32, LuaError> {
    state.dcall(c.func as usize, c.nresults)?;
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
    let zio = luaZ::Zio::new( reader, data);
    luaD::protected_parser(state, zio, name.unwrap_or("?"))
}

pub fn get_top(s: &mut LuaState) -> usize {
    s.stack.len() - s.base
}

pub fn get_global(s: &mut LuaState, name: &str) {
    get_field(s, LUA_GLOBALSINDEX, name)
}

pub fn get_field(s: &mut LuaState, index: isize, name: &str) {
    let t = s.index2adr(index).clone();
    let key = TValue::from(name);
    LuaState::get_tablev2(&mut s.stack, &t, &key, None);
}

pub fn push_value(s: &mut LuaState, index: isize) {
    s.push_value(index);
}

pub fn push_literal(s: &mut LuaState, value: &str) {
    s.push_literal(value);
}

pub fn call(s: &mut LuaState, nargs: usize, nresults: i32) -> Result<(), LuaError> {
    s.call(nargs, nresults)
}

pub fn set_field(s: &mut LuaState, idx: i32, name: &str) {
    let key=TValue::from(name);
    let value=s.stack.pop().unwrap();
    let idx = if idx < 0 && idx > LUA_REGISTRYINDEX as i32 {
        idx+1
    } else {
        idx
    };
    let t=s.index2adr(idx as isize);
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
    s.index2adr(index).get_number_value()
}

pub fn to_integer(s: &mut LuaState, index: isize) -> LuaInteger {
    s.index2adr(index).get_number_value() as LuaInteger
}

pub fn to_boolean(s: &mut LuaState, index: isize) -> bool {
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

pub fn to_pointer(s: &mut LuaState, index: isize) -> *const TValue {
    let index = if index < 0 {
        s.stack.len() - (-index) as usize
    } else {
        index as usize + s.base
    };
    &s.stack[index as usize] as *const TValue
}

pub(crate) fn concat(state: &mut LuaState, n: usize) ->Result<(),LuaError>{
    if n >= 2 {
        luaV::concat(state, n, state.stack.len() - state.base - 1)?;
    } else if n == 0 {
        // push empty string
        state.push_string("");
    } // else n == 1, nothing to do
    Ok(())
}

pub(crate) fn error(state: &mut LuaState) -> Result<(), LuaError> {
    luaG::error_msg(state)
}

pub fn create_table(state: &mut LuaState)  {
    state.stack.push(TValue::new_table());
}

pub fn set_global(state: &mut LuaState, name: &str) {
    state.set_global(name);
}

pub(crate) fn _replace(_state: &mut LuaState, _lua_environindex: isize)  {
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
        obj_index+1
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
    let o =state.index2adr(idx as isize);
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

pub fn push_rust_function(state: &mut LuaState, func: LuaRustFunction) {
    state.push_rust_function(func);
}

pub fn set_top(s: &mut LuaState, idx: i32) {
    if idx >= 0 {
        while s.stack.len() < s.base + idx as usize {
            s.push_nil();
        }
    } else {
        let newlen = s.stack.len() + 1 - (-idx) as usize;
        s.stack.resize(newlen, TValue::Nil);
    }
}

pub fn next(s: &mut LuaState, idx: i32) -> bool {
    let t=s.index2adr(idx as isize);
    if let TValue::Table(tref) = t {
        let t = tref.borrow();
        let (k,v) = t.next(s.stack.last().unwrap());
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
