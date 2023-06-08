//! Lua API

use crate::{
    luaD, luaG, luaV, luaZ,
    object::{TValue, TVALUE_TYPE_NAMES},
    state::{LuaState, PanicFunction},
    Reader, LUA_GLOBALSINDEX, LuaNumber, LUA_REGISTRYINDEX, LuaInteger,
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

impl std::fmt::Display for LuaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id=*self as isize;
        write!(f,"{}",if id == -1 {"none"} else {TVALUE_TYPE_NAMES[id as usize]})
    }
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
        debug_assert!(state.stack.len() >= nargs + 1);
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
    s.set_tablev(t, key, value);
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

pub fn push_number(s: &mut LuaState, value: LuaNumber) {
    s.push_number(value);
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


pub fn typename(_s: &LuaState, tt: LuaType) -> &str {
    let index=tt as usize;
    TVALUE_TYPE_NAMES[index]
}

pub fn to_pointer(s: &mut LuaState, index: isize) -> *const TValue {
    s.index2adr(index) as *const TValue
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

pub(crate) fn create_table(state: &mut LuaState, arg_1: i32, arg_2: i32)  {
    todo!()
}

pub(crate) fn replace(state: &mut LuaState, lua_environindex: isize)  {
    todo!()
}