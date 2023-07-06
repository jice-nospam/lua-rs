//! Some generic functions over Lua objects

use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    lex::str2d,
    limits::Instruction,
    luaH::{Table, TableRef},
    parser::UpValDesc,
    LuaFloat, LuaInteger, LuaRustFunction,
};

/// index in the current stack
pub type StkId = usize;

pub type UserDataRef = Rc<RefCell<UserData>>;
pub type ClosureRef = Rc<RefCell<Closure>>;
/// index in the LuaState.protos vector
pub type ProtoId = usize;

#[derive(Clone, Default)]
pub enum TValue {
    #[default]
    Nil,
    Float(LuaFloat),
    Integer(LuaInteger),
    String(Rc<String>),
    Table(TableRef),
    Function(ClosureRef),
    Boolean(bool),
    UserData(UserDataRef),
    Thread(),
    LightUserData(),
}
impl From<&str> for TValue {
    fn from(value: &str) -> Self {
        Self::String(Rc::new(value.to_owned()))
    }
}

impl From<String> for TValue {
    fn from(value: String) -> Self {
        Self::String(Rc::new(value))
    }
}

impl From<Closure> for TValue {
    fn from(value: Closure) -> Self {
        Self::Function(Rc::new(RefCell::new(value)))
    }
}

impl From<RClosure> for TValue {
    fn from(value: RClosure) -> Self {
        Self::Function(Rc::new(RefCell::new(Closure::Rust(value))))
    }
}

impl From<LClosure> for TValue {
    fn from(value: LClosure) -> Self {
        Self::Function(Rc::new(RefCell::new(Closure::Lua(value))))
    }
}

impl From<&TableRef> for TValue {
    fn from(value: &TableRef) -> Self {
        Self::Table(Rc::clone(value))
    }
}

impl From<LuaInteger> for TValue {
    fn from(value: LuaInteger) -> Self {
        Self::Integer(value)
    }
}

impl From<LuaFloat> for TValue {
    fn from(value: LuaFloat) -> Self {
        Self::Float(value)
    }
}

impl TValue {
    #[inline]
    pub fn get_type_name(&self) -> &str {
        match self {
            TValue::Nil => "nil",
            TValue::Float(_) | TValue::Integer(_) => "number",
            TValue::String(_) => "string",
            TValue::Table(_) => "table",
            TValue::Function(_) => "function",
            TValue::Boolean(_) => "boolean",
            TValue::UserData(_) | TValue::LightUserData() => "userdata",
            TValue::Thread() => "thread",
        }
    }
    pub fn new_table() -> Self {
        Self::Table(Rc::new(RefCell::new(Table::new())))
    }
    pub fn is_nil(&self) -> bool {
        matches!(self, TValue::Nil)
    }
    pub fn get_float_value(&self) -> LuaFloat {
        match self {
            TValue::Float(n) => *n,
            _ => 0.0,
        }
    }
    pub fn get_integer_value(&self) -> LuaInteger {
        match self {
            TValue::Integer(n) => *n,
            _ => 0,
        }
    }
    pub fn borrow_string_value(&self) -> &str {
        match self {
            TValue::String(s) => &s,
            _ => unreachable!(),
        }
    }
    pub fn is_float(&self) -> bool {
        matches!(self, TValue::Float(_))
    }
    pub fn is_integer(&self) -> bool {
        matches!(self, TValue::Integer(_))
    }
    pub fn is_number(&self) -> bool {
        match self {
            TValue::Float(_) | TValue::Integer(_) => true,
            _ => false,
        }
    }
    pub fn to_float(&self) -> Result<LuaFloat, ()> {
        match self {
            TValue::Float(n) => Ok(*n),
            _ => Err(()),
        }
    }
    pub fn to_integer(&self) -> Result<LuaInteger, ()> {
        match self {
            TValue::Integer(n) => Ok(*n),
            _ => Err(()),
        }
    }
    pub fn into_float(&self) -> Result<LuaFloat, ()> {
        match self {
            TValue::Integer(n) => Ok(*n as LuaFloat),
            TValue::Float(n) => Ok(*n),
            TValue::String(rcs) => str2d(&*rcs).ok_or(()),
            _ => Err(()),
        }
    }
    pub fn into_integer(&self) -> Result<LuaInteger, ()> {
        match self {
            TValue::Integer(n) => Ok(*n),
            TValue::Float(n) => {
                if n.fract() == 0.0 {
                    Ok(*n as LuaInteger)
                } else {
                    Err(())
                }
            }
            TValue::String(rcs) => {
                let v = str2d(&*rcs).ok_or(())?;
                if v.fract() == 0.0 {
                    Ok(v as LuaInteger)
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        }
    }
    pub fn is_string(&self) -> bool {
        matches!(self, TValue::String(_))
    }
    pub fn is_table(&self) -> bool {
        matches!(self, TValue::Table(_))
    }
    pub fn is_function(&self) -> bool {
        matches!(self, TValue::Function(_))
    }
    pub fn is_boolean(&self) -> bool {
        matches!(self, TValue::Boolean(_))
    }

    pub(crate) fn is_false(&self) -> bool {
        match self {
            TValue::Nil => true,
            TValue::Boolean(b) => !*b,
            _ => false,
        }
    }
    pub(crate) fn to_pointer(&self) -> *const TValue {
        self as *const TValue
    }
}

impl Display for TValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TValue::Nil => write!(f, "nil"),
            TValue::Float(n) => write!(f, "{}", n),
            TValue::Integer(i) => write!(f, "{}", i),
            TValue::Boolean(b) => write!(f, "{}", b),
            TValue::String(s) => write!(f, "{}", s),
            _ => write!(f, "{} : {:?}", self.get_type_name(), self.to_pointer()),
        }
    }
}

impl std::fmt::Debug for TValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TValue::String(s) => write!(f, "{:?}", s),
            _ => write!(f, "{}", self),
        }
    }
}

impl PartialEq for TValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Float(l0), Self::Float(r0)) => l0 == r0,
            (Self::Integer(l0), Self::Integer(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Eq for TValue {}

impl std::hash::Hash for TValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            TValue::String(s) => s.hash(state),
            _ => core::mem::discriminant(self).hash(state),
        }
    }
}

#[derive(Clone, Default)]
pub struct UserData {
    pub metatable: Option<TableRef>,
    pub env: Option<TableRef>,
}

#[derive(Clone, Default)]
pub struct LocVar {
    pub name: String,
    /// first point where variable is active
    pub start_pc: usize,
    /// first point where variable is dead
    pub end_pc: usize,
}

#[derive(Clone, Default)]
pub struct Proto {
    /// constants used by the function
    pub k: Vec<TValue>,
    /// the bytecode
    pub code: Vec<Instruction>,
    /// functions defined inside the function
    pub p: Vec<ProtoId>,
    /// map from opcodes to source lines
    pub lineinfo: Vec<usize>,
    /// information about local variables
    pub locvars: Vec<LocVar>,
    /// upvalues information
    pub upvalues: Vec<UpValDesc>,
    /// file name
    pub source: String,
    pub linedefined: usize,
    pub lastlinedefined: usize,
    /// number of fixed parameters
    pub numparams: usize,
    pub is_vararg: bool,
    /// maximum stack used by this function
    pub maxstacksize: usize,
}

impl Proto {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_owned(),
            // registers 0/1 are always valid
            maxstacksize: 2,
            ..Self::default()
        }
    }

    pub(crate) fn next_pc(&self) -> i32 {
        self.code.len() as i32
    }
}

#[derive(Clone, Default)]
pub struct UpVal {
    pub v: StkId,
    pub value: TValue,
}

/// native rust closure
#[derive(Clone)]
pub struct RClosure {
    pub f: LuaRustFunction,
    pub upvalues: Vec<TValue>,
}

impl RClosure {
    pub fn new(func: LuaRustFunction) -> Self {
        Self {
            f: func,
            upvalues: Vec::new(),
        }
    }
    pub fn borrow_upvalue(&self, index: usize) -> &TValue {
        &self.upvalues[index]
    }
}

/// Lua closure
#[derive(Clone)]
pub struct LClosure {
    pub proto: ProtoId,
    pub upvalues: Vec<UpVal>,
}

impl LClosure {
    pub fn new(proto: ProtoId, nupval: usize) -> Self {
        let mut upvalues = Vec::new();
        for _ in 0..nupval {
            upvalues.push(UpVal::default())
        }
        Self { proto, upvalues }
    }
}

#[derive(Clone)]
pub enum Closure {
    Rust(RClosure),
    Lua(LClosure),
}

impl Closure {
    #[inline]
    pub fn add_lua_upvalue(&mut self, upval: UpVal) {
        if let Closure::Lua(cl) = self {
            cl.upvalues.push(upval);
            return;
        }
        unreachable!()
    }
    #[inline]
    pub fn set_lua_upvalue(&mut self, id: usize, upval: UpVal) {
        if let Closure::Lua(cl) = self {
            cl.upvalues[id] = upval;
            return;
        }
        unreachable!()
    }
    #[inline]
    pub fn borrow_lua_upval(&self, id: usize) -> &UpVal {
        if let Closure::Lua(cl) = self {
            return &cl.upvalues[id];
        }
        unreachable!()
    }
    #[inline]
    pub fn get_lua_upvalue(&self, id: usize) -> TValue {
        if let Closure::Lua(cl) = self {
            return cl.upvalues[id].value.clone();
        }
        unreachable!()
    }
    #[inline]
    pub fn get_rust_upvalue(&self, id: usize) -> TValue {
        if let Closure::Rust(cl) = self {
            return cl.upvalues[id].clone();
        }
        unreachable!()
    }
    #[inline]
    pub fn set_lua_upval_value(&mut self, id: usize, value: TValue) {
        if let Closure::Lua(cl) = self {
            cl.upvalues[id].value = value;
            return;
        }
        unreachable!()
    }
    #[inline]
    pub fn get_lua_upval_desc(&self, id: usize) -> UpVal {
        if let Closure::Lua(cl) = self {
            return cl.upvalues[id].clone();
        }
        unreachable!()
    }
    #[inline]
    pub fn get_proto_id(&self) -> usize {
        match self {
            Closure::Rust(_cl) => unreachable!(),
            Closure::Lua(cl) => cl.proto,
        }
    }
    pub fn get_nupvalues(&self) -> usize {
        match self {
            Closure::Rust(cl) => cl.upvalues.len(),
            Closure::Lua(cl) => cl.upvalues.len(),
        }
    }
    pub fn borrow_lua_closure(&self) -> &LClosure {
        if let Closure::Lua(lcl) = self {
            lcl
        } else {
            unreachable!()
        }
    }
}

/// identify current chunkid (file name or source code)
pub fn chunk_id(source_name: &str) -> String {
    if let Some(stripped) = source_name.strip_prefix('=') {
        stripped.to_owned()
    } else if let Some(stripped) = source_name.strip_prefix('@') {
        format!("{}...", stripped)
    } else {
        // get first line of source code
        let len = source_name.find(['\r', '\n']).unwrap_or(source_name.len());
        format!("[string \"{}...\"]", &source_name[0..len])
    }
}

/// converts an integer to a "floating point byte", represented as
/// (eeeeexxx), where the real value is (1xxx) * 2^(eeeee - 1) if
/// eeeee != 0 and (xxx) otherwise.
pub(crate) const fn int2fb(val: u32) -> u32 {
    let mut e = 0; // exponent
    let mut val = val;
    while val >= 16 {
        val = (val + 1) >> 1;
        e += 1;
    }
    if val < 8 {
        val
    } else {
        ((e + 1) << 3) | (val - 8)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::TValue;
    use crate::{luaL, state::LuaState};
    #[test]
    /// check if the TValue::Table works
    fn table() {
        let mut state = luaL::newstate();
        let t = TValue::new_table();
        state.set_tablev(&t, TValue::from("key"), TValue::from("value"));
        LuaState::get_tablev2(&mut state.stack, &t, &TValue::from("key"), None);
        let v = &state.stack[state.stack.len() - 1];

        assert!(if let TValue::String(s) = v {
            if **s == "value" {
                true
            } else {
                false
            }
        } else {
            false
        });
    }

    #[test]
    /// check if TValue can be used as HashMap keys
    fn hashmap() {
        let mut h = HashMap::new();
        let k = TValue::from("key");
        h.insert(k, 123);
        let v = h.get(&TValue::from("key"));

        assert_eq!(v, Some(&123));
    }
}
