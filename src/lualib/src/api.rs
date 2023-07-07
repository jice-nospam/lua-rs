//! Lua API

use std::rc::Rc;

use crate::{
    luaD, luaG,
    luaV::{self, f_close, CLOSEKTOP},
    luaZ,
    object::{Closure, TValue},
    opcodes::OpCode,
    state::{LuaState, PanicFunction},
    LuaFloat, LuaInteger, LuaRustFunction, Reader, LUA_REGISTRYINDEX, LUA_RIDX_GLOBALS,
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

struct CallData {
    func: u32,
    nresults: i32,
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

/// Performs an arithmetic or bitwise operation over the two values
/// (or one, in the case of negations) at the top of the stack,
/// with the value on the top being the second operand,
/// pops these values, and pushes the result of the operation.
/// The function follows the semantics of the corresponding Lua operator
/// (that is, it may call metamethods).
/// The value of op must be one of the following constants:
/// * OpCode::Add : performs addition (+)
/// * OpCode::Sub : performs subtraction (-)
/// * OpCode::Mul : performs multiplication (*)
/// * OpCode::Div : performs float division (/)
/// * OpCode::IntegerDiv : performs floor division(//)
/// * OpCode::Mod : performs modulo (%)
/// * OpCode::Pow : performs exponentiation (^)
/// * OpCode::UnaryMinus : performs mathematical negation (unary -)
/// * OpCode::BinaryNot : performs bitwise NOT (~)
/// * OpCode::BinaryAnd : performs bitwise AND (&)
/// * OpCode::BinaryOr : performs bitwise OR (|)
/// * OpCode::BinaryXor : performs bitwise exclusive OR (~)
/// * OpCode::Shl : performs left shift (<<)
/// * OpCode::Shr : performs right shift (>>)
pub fn arith(s: &mut LuaState, op: OpCode) {
    if op == OpCode::UnaryMinus || op == OpCode::BinaryNot {
        // for unary operations, add fake 2nd operand
        s.stack.push(s.stack.last().unwrap().clone());
    }
    let idx = s.stack.len() - 2;
    let v1 = s.stack[idx].clone();
    let v2 = s.stack[idx + 1].clone();
    crate::code::arith(op, &v1, &v2, &mut s.stack[idx]);
    s.stack.pop(); // remove second operand
}

/// Sets a new panic function and returns the old one
pub fn at_panic(state: &mut LuaState, panic: PanicFunction) -> Option<PanicFunction> {
    let old = state.g.panic.take();
    state.g.panic = Some(panic);
    old
}

///  Calls a function. Like regular Lua calls, lua_call respects the __call metamethod.
/// So, here the word "function" means any callable value.
///
/// To do a call you must use the following protocol:
/// - first, the function to be called is pushed onto the stack;
/// - then, the arguments to the call are pushed in direct order;
/// that is, the first argument is pushed first.
/// - Finally you call lua::call;
/// nargs is the number of arguments that you pushed onto the stack.
/// When the function returns, all arguments and the function value are popped
/// and the call results are pushed onto the stack.
/// The number of results is adjusted to nresults, unless nresults is LUA_MULTRET.
/// In this case, all results from the function are pushed;
/// The function results are pushed onto the stack in direct order
/// (the first result is pushed first), so that after the call the last result is
/// on the top of the stack.
pub fn call(s: &mut LuaState, nargs: usize, nresults: i32) -> Result<(), LuaError> {
    s.call(nargs, nresults, 0, None)
}

/// Compares two Lua values.
/// Returns true if the value at index index1 satisfies op when compared
/// with the value at index index2, following the semantics of the corresponding
/// Lua operator (that is, it may call metamethods).
/// Otherwise returns false.
/// Also returns false if any of the indices is not valid.
/// The value of op must be one of the following constants:
/// OpCode::Eq : compares for equality (==)
/// OpCode::Lt : compares for less than (<)
/// OpCode::Le : compares for less or equals (<=)
pub fn compare(
    s: &mut LuaState,
    index1: isize,
    index2: isize,
    op: OpCode,
) -> Result<bool, LuaError> {
    if !(s.is_index_valid(index1) && s.is_index_valid(index2)) {
        return Ok(false);
    }
    let o1 = s.index2adr(index1);
    let o2 = s.index2adr(index2);
    match op {
        OpCode::Eq => Ok(o1 == o2),
        OpCode::Lt => Ok(less_than(s, &o1, &o2)),
        OpCode::Le => Ok(less_equal(s, &o1, &o2)),
        _ => Ok(false),
    }
}

fn less_than(_s: &mut LuaState, o1: &TValue, o2: &TValue) -> bool {
    if o1.is_number() && o2.is_number() {
        o1.into_float().unwrap() < o2.into_float().unwrap()
    } else if let (TValue::String(sa), TValue::String(sb)) = (o1, o2) {
        sa < sb
    } else {
        // TODO metamethods
        todo!()
    }
}

fn less_equal(_s: &mut LuaState, o1: &TValue, o2: &TValue) -> bool {
    if o1.is_number() && o2.is_number() {
        o1.into_float().unwrap() <= o2.into_float().unwrap()
    } else if let (TValue::String(sa), TValue::String(sb)) = (o1, o2) {
        sa <= sb
    } else {
        // TODO metamethods
        todo!()
    }
}

/// Concatenates the n values at the top of the stack, pops them, and leaves the result on the top.
/// If n is 1, the result is the single value on the stack
/// (that is, the function does nothing);
/// if n is 0, the result is the empty string.
/// Concatenation is performed following the usual semantics of Lua
pub fn concat(state: &mut LuaState, n: usize) -> Result<(), LuaError> {
    if n >= 2 {
        luaV::concat(state, n)?;
    } else if n == 0 {
        // push empty string
        state.push_string("");
    } // else n == 1, nothing to do
    Ok(())
}

/// Copies the element at index from_idx into the valid index to_idx, replacing the value at that position. Values at other positions are not affected.
pub fn copy(s: &mut LuaState, from_idx: isize, to_idx: isize) {
    let to = s.index2adr(to_idx).clone();
    s.set_index(from_idx, to);
}

/// Creates a new empty table and pushes it onto the stack.
/// Parameter narr is a hint for how many elements the table will have as a sequence;
/// parameter nrec is a hint for how many other elements the table will have.
/// Lua may use these hints to preallocate memory for the new table.
/// This preallocation may help performance when you know in advance how many elements
/// the table will have. Otherwise you can use the function lua::new_table.
pub fn create_table(state: &mut LuaState, narr: usize, nrec: usize) {
    state.create_table(narr, nrec);
}

/// Raises a Lua error, using the value on the top of the stack as the error object.
pub fn error(state: &mut LuaState) -> Result<(), LuaError> {
    luaG::error_msg(state)
}

/// Pushes onto the stack the value t[k], where t is the value at the given index.
/// As in Lua, this function may trigger a metamethod for the "index" event
pub fn get_field(s: &mut LuaState, index: isize, name: &str) {
    let t = s.index2adr(index).clone();
    let key = TValue::from(name);
    LuaState::get_tablev2(&mut s.stack, &t, &key, None);
}

/// Pushes onto the stack the value of the global name.
pub fn get_global(s: &mut LuaState, name: &str) {
    let gt = s.get_global_table();
    let top = s.stack.len();
    s.stack.push(TValue::from(name));
    LuaState::get_tablev2(&mut s.stack, &gt, &TValue::from(name), Some(top));
}

/// Pushes onto the stack the value t[i], where t is the value at the given index.
/// As in Lua, this function may trigger a metamethod for the "index" event
pub fn get_i(s: &mut LuaState, index: isize, i: LuaInteger) {
    let t = s.index2adr(index).clone();
    let key = TValue::from(i);
    LuaState::get_tablev2(&mut s.stack, &t, &key, None);
}

/// If the value at the given index has a metatable, the function pushes that metatable
/// onto the stack and returns true.
/// Otherwise, the function returns false and pushes nothing on the stack.
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

/// Pushes onto the stack the value t[k], where t is the value at the given index
/// and k is the value on the top of the stack.
///
/// This function pops the key from the stack, pushing the resulting value in its place.
/// As in Lua, this function may trigger a metamethod for the "index" event
pub fn get_table(s: &mut LuaState, index: isize) {
    let t = s.index2adr(index).clone();
    let key = s.stack.pop().unwrap();
    LuaState::get_tablev2(&mut s.stack, &t, &key, None);
}

/// Returns the index of the top element in the stack.
/// Because indices start at 1, this result is equal to the number of
/// elements in the stack (and so 0 means an empty stack).
pub fn get_top(s: &mut LuaState) -> usize {
    s.stack.len() - (s.base_ci[s.ci].func + 1)
}

/// Moves the top element into the given valid index,
/// shifting up the elements above this index to open space.
/// This function cannot be called with a pseudo-index,
/// because a pseudo-index is not an actual stack position.
pub fn insert(s: &mut LuaState, index: isize) {
    if index.abs() as usize <= s.stack.len() {
        let o = s.stack.pop().unwrap();
        let index = s.index2stack(index).unwrap();
        s.stack.insert(index, o);
    }
}

/// Returns true if the value at the given index is a boolean,
/// and false otherwise.
pub fn is_boolean(s: &mut LuaState, index: isize) -> bool {
    s.index2adr(index).is_boolean()
}

/// Returns true if the value at the given index is a Rust function, and false otherwise.
pub fn is_rust_function(s: &mut LuaState, index: isize) -> bool {
    s.index2adr(index).is_rust_function()
}

/// Returns true if the value at the given index is a function
/// (either Rust or Lua), and false otherwise.
pub fn is_function(s: &mut LuaState, index: isize) -> bool {
    s.index2adr(index).is_function()
}

/// Returns true if the value at the given index is an integer (that is, the value is a number and is represented as an integer), and false otherwise.
pub fn is_integer(s: &mut LuaState, index: isize) -> bool {
    matches!(s.index2adr(index), TValue::Integer(_))
}

/// Returns true if the value at the given index is nil, and false otherwise.
pub fn is_nil(s: &mut LuaState, index: isize) -> bool {
    s.index2adr(index).is_nil()
}

/// Returns true if the given index is not valid, and false otherwise.
pub fn is_none(s: &mut LuaState, index: isize) -> bool {
    !s.is_index_valid(index)
}

/// Returns true if the given index is not valid or if the value at this index is nil, and false otherwise.
pub fn is_none_or_nil(s: &mut LuaState, index: isize) -> bool {
    is_none(s, index) || is_nil(s, index)
}

/// Returns true if the value at the given index is a number or a string convertible to a number, and false otherwise.
pub fn is_number(s: &mut LuaState, index: isize) -> bool {
    s.index2adr(index).into_float().is_some()
}

/// Returns true if the value at the given index is a string
/// or a number (which is always convertible to a string), and false otherwise.
pub fn is_string(s: &mut LuaState, index: isize) -> bool {
    matches!(
        s.index2adr(index),
        TValue::Float(_) | TValue::Integer(_) | TValue::String(_)
    )
}

/// Returns true if the value at the given index is a table, and false otherwise.
pub fn is_table(s: &mut LuaState, index: isize) -> bool {
    s.index2adr(index).is_table()
}

/// Returns the length of the value at the given index.
/// It is equivalent to the '#' operator in Lua and may trigger
/// a metamethod for the "length" event.
/// The result is pushed on the stack.
pub fn len(s: &mut LuaState, index: isize) -> usize {
    let o = s.index2adr(index);
    if let Some(l) = o.try_len() {
        s.push_integer(l as LuaInteger);
        l
    } else {
        // TODO metamethod
        todo!()
    }
}

/// Loads a Lua chunk without running it.
/// If there are no errors, load pushes the compiled chunk
/// as a Lua function on top of the stack.
/// Otherwise, it pushes an error message.
///
/// The load function uses a user-supplied reader function
/// to read the chunk (see Reader).
/// The data argument is an opaque value passed to the reader
/// function.
///
/// The chunkname argument gives a name to the chunk,
/// which is used for error messages and in debug information
///
/// load automatically detects whether the chunk is text
/// or binary and loads it accordingly.
///
/// lua_load uses the stack internally, so the reader function
/// must always leave the stack unmodified when returning.
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
                if !lcl.upvalues.is_empty() {
                    // does it have one upvalue?
                    let gt = state.get_global_table();
                    // set global table as 1st upvalue of 'lcl' (may be LUA_ENV)
                    lcl.upvalues[0].value = gt.clone();
                }
            }
        }
    }
    res
}

pub fn to_string(state: &mut LuaState, idx: isize) -> Option<String> {
    // TODO convert in stack
    match state.index2adr(idx) {
        TValue::String(s) => Some(s.as_ref().clone()),
        TValue::Float(n) => Some(format!("{}", n)),
        TValue::Integer(n) => Some(format!("{}", n)),
        _ => None,
    }
}

fn f_call(state: &mut LuaState, c: &CallData) -> Result<i32, LuaError> {
    state.dcall_no_yield(c.func as usize, c.nresults)?;
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

/// Pops a value from the stack and sets it as the new value of global name.
pub fn set_global(state: &mut LuaState, name: &str) {
    let gt = state.get_global_table();
    let key = TValue::from(name);
    let value = state.stack.pop().unwrap();
    LuaState::set_tablev(state, &gt, key, value);
}

/// Pushes a copy of the element at the given index onto the stack.
pub fn push_value(s: &mut LuaState, index: isize) {
    s.push_value(index);
}

/// Pushes the string `value` onto the stack.
pub fn push_literal(s: &mut LuaState, value: &str) {
    s.push_literal(value);
}

pub fn set_field(s: &mut LuaState, idx: isize, name: &str) {
    let key = TValue::from(name);
    let value = s.stack.pop().unwrap();
    let idx = if idx < 0 && idx > LUA_REGISTRYINDEX {
        idx + 1
    } else {
        idx
    };
    let t = s.index2adr(idx);
    s.set_tablev(&t, key, value);
}

pub fn pop(s: &mut LuaState, count: usize) {
    s.pop_stack(count);
}

pub fn push_string(s: &mut LuaState, value: &str) {
    s.push_string(value);
}

pub fn push_number(s: &mut LuaState, value: LuaFloat) {
    s.push_number(value);
}

pub fn push_integer(s: &mut LuaState, value: LuaInteger) {
    s.push_integer(value);
}

pub fn push_boolean(s: &mut LuaState, value: bool) {
    s.push_boolean(value);
}

pub fn push_nil(s: &mut LuaState) {
    s.push_nil();
}

pub fn to_number(s: &mut LuaState, index: isize) -> Option<LuaFloat> {
    // TODO convert in stack
    s.index2adr(index).into_float()
}

pub fn to_integer(s: &mut LuaState, index: isize) -> Option<LuaInteger> {
    // TODO convert in stack
    s.index2adr(index).into_integer()
}

pub fn to_boolean(s: &mut LuaState, index: isize) -> bool {
    // TODO convert in stack
    s.index2adr(index).is_false()
}

pub fn to_pointer(s: &mut LuaState, index: isize) -> *const TValue {
    s.index2adr(index).to_pointer()
}

pub(crate) fn _replace(_state: &mut LuaState, _lua_environindex: isize) {
    todo!()
}

pub(crate) fn set_metatable(state: &mut LuaState, obj_index: i32) {
    let mt = state.stack.pop().unwrap();
    let mt = if mt.is_nil() {
        None
    } else if let TValue::Table(tref) = mt {
        Some(tref)
    } else {
        unreachable!()
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

pub(crate) fn raw_get_i(state: &mut LuaState, idx: isize, n: usize) {
    let o = state.index2adr(idx);
    if let TValue::Table(tref) = o {
        let value = {
            let mut t = tref.borrow_mut();
            t.get_num(n).clone()
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
    let func = s.base_ci[s.ci].func;
    let diff = if idx >= 0 {
        let diff = (func as i32 + 1 + idx) - s.stack.len() as i32;
        for _ in 0..diff {
            s.push_nil();
        }
        diff
    } else {
        idx + 1
    };
    let new_top = (s.stack.len() as i32 + diff) as usize;
    if diff < 0 && !s.tbc_list.is_empty() && *s.tbc_list.last().unwrap() >= new_top {
        f_close(s, new_top, CLOSEKTOP, false).ok();
    }
    s.stack.resize(new_top, TValue::Nil);
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
    raw_get_i(state, LUA_REGISTRYINDEX, LUA_RIDX_GLOBALS);
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
    s.new_table();
}

pub fn number_to_integer(v: LuaFloat) -> LuaInteger {
    v as LuaInteger
}
