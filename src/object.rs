//! Some generic functions over Lua objects

use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    limits::Instruction,
    luaH::{Table, TableRef},
    LuaNumber, LuaRustFunction, LuaType,
};

/// index in the current stack
pub type StkId = usize;

pub type UserDataRef = Rc<RefCell<UserData>>;
/// index in the LuaState.protos vector
pub type ProtoRef = usize;

#[derive(Clone, Default)]
pub enum TValue {
    #[default]
    Nil,
    Number(LuaNumber),
    String(Rc<String>),
    Table(TableRef),
    Function(Rc<Closure>),
    Boolean(bool),
    UserData(UserDataRef),
    Thread(),
    LightUserData(),
}

pub(crate) const TVALUE_TYPE_NAMES: [&str; 8] = [
    "nil", "number", "string", "table", "function", "userdata", "thread", "userdata",
];

pub const TVALUE_TYPE_COUNT: usize = 9;

impl From<&str> for TValue {
    fn from(value: &str) -> Self {
        Self::String(Rc::new(value.to_owned()))
    }
}

impl From<Closure> for TValue {
    fn from(value: Closure) -> Self {
        Self::Function(Rc::new(value))
    }
}

impl From<RClosure> for TValue {
    fn from(value: RClosure) -> Self {
        Self::Function(Rc::new(Closure::Rust(value)))
    }
}

impl From<LClosure> for TValue {
    fn from(value: LClosure) -> Self {
        Self::Function(Rc::new(Closure::Lua(value)))
    }
}

impl From<&TableRef> for TValue {
    fn from(value: &TableRef) -> Self {
        Self::Table(Rc::clone(value))
    }
}

impl TValue {
    pub fn get_lua_type(&self) -> LuaType {
        match self {
            TValue::Boolean(_) => LuaType::Boolean,
            TValue::Nil => LuaType::Nil,
            TValue::Number(_) => LuaType::Number,
            TValue::String(_) => LuaType::String,
            TValue::Table(_) => LuaType::Table,
            TValue::Function(_) => LuaType::Function,
            TValue::UserData(_) => LuaType::UserData,
            TValue::Thread() => LuaType::Thread,
            TValue::LightUserData() => LuaType::LightUserData,
        }
    }
    pub fn get_lua_closure(&self) -> &LClosure {
        if let TValue::Function(cl) = self {
            if let Closure::Lua(luacl) = cl.as_ref() {
                return luacl;
            }
        }
        unreachable!()
    }
    #[inline]
    pub const fn get_type_name(&self) -> &str {
        TVALUE_TYPE_NAMES[self.type_as_usize()]
    }
    pub const fn type_as_usize(&self) -> usize {
        match self {
            TValue::Nil => 0,
            TValue::Number(_) => 1,
            TValue::String(_) => 2,
            TValue::Table(_) => 3,
            TValue::Function(_) => 4,
            TValue::Boolean(_) => 5,
            TValue::UserData(_) => 6,
            TValue::Thread() => 7,
            TValue::LightUserData() => 8,
        }
    }
    pub fn new_table() -> Self {
        Self::Table(Rc::new(RefCell::new(Table::new())))
    }
    pub fn is_nil(&self) -> bool {
        match self {
            TValue::Nil => true,
            _ => false,
        }
    }
    pub fn get_number_value(&self) -> LuaNumber {
        match self {
            TValue::Number(n) => *n,
            _ => 0.0,
        }
    }
    pub fn is_number(&self) -> bool {
        match self {
            TValue::Number(_) => true,
            _ => false,
        }
    }
    pub fn is_string(&self) -> bool {
        match self {
            TValue::String(_) => true,
            _ => false,
        }
    }
    pub fn is_table(&self) -> bool {
        match self {
            TValue::Table(_) => true,
            _ => false,
        }
    }
    pub fn is_function(&self) -> bool {
        match self {
            TValue::Function(_) => true,
            _ => false,
        }
    }
    pub fn is_boolean(&self) -> bool {
        match self {
            TValue::Boolean(_) => true,
            _ => false,
        }
    }

    pub(crate) fn is_false(&self) -> bool {
        match self {
            TValue::Nil => true,
            TValue::Boolean(b) => *b,
            _ => false,
        }
    }
}

impl Display for TValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TValue::Nil => write!(f, "nil"),
            TValue::Number(n) => write!(f, "{}", n),
            TValue::Boolean(b) => write!(f, "{}", b),
            TValue::String(s) => write!(f, "{}", s),
            TValue::Table(tr) => write!(f, "table: {:p}", tr),
            TValue::Function(cl) => write!(f, "function: {:p}", cl),
            TValue::UserData(_) => todo!(),
            TValue::Thread() => todo!(),
            TValue::LightUserData() => todo!(),
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
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
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
    pub p: Vec<ProtoRef>,
    /// map from opcodes to source lines
    pub lineinfo: Vec<usize>,
    /// information about local variables
    pub locvars: Vec<LocVar>,
    /// number of upvalues
    pub nups: usize,
    pub linedefined: usize,
    pub lastlinedefined: usize,
    pub numparams: usize,
    pub is_vararg: bool,
    pub maxstacksize: usize,
    /// file name
    pub source: String,
}

impl Proto {
    pub fn new(source: &str) -> Self {
        Self {
            source:source.to_owned(),
            ..Self::default()
        }
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
    pub env: TableRef,
    pub envvalue: TValue,
}

impl RClosure {
    pub fn new(func: LuaRustFunction, env: TableRef) -> Self {
        let envvalue = TValue::Table(Rc::clone(&env));
        Self {
            f: func,
            upvalues: Vec::new(),
            env,
            envvalue,
        }
    }
    pub fn borrow_upvalue(&self, index: usize) -> &TValue {
        &self.upvalues[index]
    }
}

/// Lua closure
#[derive(Clone)]
pub struct LClosure {
    pub proto: ProtoRef,
    pub upvalues: Vec<UpVal>,
    pub env: TableRef,
    pub envvalue: TValue,
}

impl LClosure {
    pub fn new(proto: ProtoRef, env: TableRef) -> Self {
        let envvalue = TValue::Table(Rc::clone(&env));
        Self {
            proto,
            upvalues: Vec::new(),
            env,
            envvalue,
        }
    }
}

#[derive(Clone)]
pub enum Closure {
    Rust(RClosure),
    Lua(LClosure),
}

impl Closure {
    pub fn get_env(&self) -> TableRef {
        match self {
            Closure::Rust(cl) => Rc::clone(&cl.env),
            Closure::Lua(cl) => Rc::clone(&cl.env),
        }
    }

    #[inline]
    pub fn get_lua_upvalue(&self, id: usize) -> TValue {
        if let Closure::Lua(cl) = self {
            return cl.upvalues[id].value.clone();
        }
        unreachable!()
    }
    pub fn get_proto_id(&self) -> usize {
        match self {
            Closure::Rust(cl) => unreachable!(),
            Closure::Lua(cl) => cl.proto,
        }
    }
    pub fn get_envvalue(&self) -> &TValue {
        match self {
            Closure::Rust(cl) => &cl.envvalue,
            Closure::Lua(cl) => &cl.envvalue,
        }
    }
    pub fn get_nupvalues(&self) -> usize {
        match self {
            Closure::Rust(cl) => cl.upvalues.len(),
            Closure::Lua(cl) => cl.upvalues.len(),
        }
    }
}

/// identify current chunkid (file name or source code)
pub fn chunk_id(source_name: &str) -> String {
    if source_name.starts_with('=') {
        source_name[1..].to_owned()
    } else if source_name.starts_with('@') {
        format!("{}...", &source_name[1..])
    } else {
        // get first line of source code
        let len = source_name.find(&['\r', '\n']).unwrap_or(source_name.len());
        format!("[string \"{}...\"]", &source_name[0..len])
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

/// converts an integer to a "floating point byte", represented as
/// (eeeeexxx), where the real value is (1xxx) * 2^(eeeee - 1) if
/// eeeee != 0 and (xxx) otherwise.
pub(crate) const fn INT2FB(val: u32) -> u32 {
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
