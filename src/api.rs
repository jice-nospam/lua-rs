//! Lua API

use std::rc::Rc;

use crate::{
    luaD, luaZ,
    object::{TValue, TVALUE_TYPE_NAMES},
    state::{LuaState, PanicFunction},
    LuaStateRef, Reader, LUA_GLOBALSINDEX,
};

#[derive(Debug)]
pub enum LuaError {
    /// error during error handling
    ErrorHandlerError,
    /// error during a function execution
    RuntimeError,
    /// error during parsing of the source code
    SyntaxError,
}

#[derive(Clone,Copy)]
pub enum LuaType {
    None = -1,
    Nil,
    Boolean,
    LightUserData,
    Number,
    String,
    Table,
    Function,
    UserData,
    Thread
}

pub fn at_panic(state: LuaStateRef, panic: PanicFunction) -> Option<PanicFunction> {
    let mut mstate = state.borrow_mut();
    let old = mstate.g.panic.take();
    mstate.g.panic = Some(panic);
    old
}

pub fn to_string(state: &mut LuaState, idx: i32) -> String {
    let idx = if idx < 0 {
        state.stack.len() - (-idx as usize)
    } else {
        idx as usize
    };
    match &state.stack[(idx - 1) as usize] {
        TValue::String(s) => s.as_ref().clone(),
        x => format!("{}", x),
    }
}

struct CallData {
    func: u32,
    nresults: i32,
}

fn f_call(stateref: LuaStateRef, c: &CallData) -> Result<i32, LuaError> {
    stateref.borrow_mut().dcall(c.func as usize, c.nresults)?;
    Ok(0)
}

pub fn pcall(
    stateref: LuaStateRef,
    nargs: usize,
    nresults: i32,
    errfunc: u32,
) -> Result<i32, LuaError> {
    let (c, func) = {
        let state = stateref.borrow_mut();
        debug_assert!(state.stack.len() >= nargs + 1);
        state.check_results(nargs, nresults);
        let c = CallData {
            func: (state.stack.len() - (nargs + 1)) as u32,
            nresults,
        };
        (c, errfunc)
    };
    let status = luaD::pcall(stateref.clone(), f_call, &c, c.func as usize, func as usize)?;
    stateref.borrow_mut().adjust_results(nresults);
    Ok(status)
}

pub fn load<T>(
    state: LuaStateRef,
    reader: Reader<T>,
    data: T,
    name: Option<&str>,
) -> Result<i32, LuaError> {
    let zio = luaZ::Zio::new(Rc::clone(&state), reader, data);
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
    let key = TValue::new_string(name);
    s.get_tablev(&t, &key, None);
}

pub fn push_value(s: &mut LuaState, index: isize) {
    s.push_value(index);
}

pub fn call(s: &mut LuaState, nargs: usize, nresults: i32) -> Result<(), LuaError> {
    s.call(nargs, nresults)
}

pub fn pop(s: &mut LuaState, count: usize) {
    s.pop_stack(count);
}

pub fn get_type(s: &LuaState, index: isize) -> LuaType {
    s.index2adr(index).get_lua_type()
}

pub fn push_string(s: &mut LuaState, value: &str) {
    s.push_string(value);
}

pub fn to_boolean(s: &mut LuaState, index: isize) -> bool {
    s.index2adr(index).is_false()
}

pub fn typename(_s: &LuaState, tt: LuaType) -> &str {
    let index=tt as usize;
    TVALUE_TYPE_NAMES[index]
}

pub fn to_pointer(s: &mut LuaState, index: isize) -> *const TValue {
    s.index2adr(index) as *const TValue
}