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
    pub fn get_type_name(&self) -> &str {
        TVALUE_TYPE_NAMES[self.type_as_usize()]
    }
    pub fn type_as_usize(&self) -> usize {
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
    pub fn new_string(val: &str) -> Self {
        Self::String(Rc::new(val.to_owned()))
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
            _ => todo!(),
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
    pub p: Vec<Proto>,
    /// map from opcodes to source lines
    pub lineinfo: Vec<usize>,
    /// information about local variables
    pub locvars: Vec<LocVar>,
    pub sizeupvalues: usize,
    pub sizek: usize,
    pub sizecode: usize,
    pub sizelineinfo: usize,
    /// size of p
    pub sizep: usize,
    pub sizelocvars: usize,
    pub linedefined: usize,
    pub lastlinedefined: usize,
    ///  number of upvalues
    pub nups: usize,
    pub numparams: usize,
    pub is_vararg: bool,
    pub maxstacksize: usize,
}

impl Proto {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Default)]
pub struct UpVal {
    pub v: TValue,
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
#[derive(Clone, Default)]
pub struct LClosure {
    pub proto: Proto,
    pub upvalues: Vec<UpVal>,
    pub env: TableRef,
    pub envvalue: TValue,
}

impl LClosure {
    pub fn new(proto: Proto, env: TableRef) -> Self {
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
    pub fn get_lua_constant(&self, id: usize) -> &TValue {
        if let Closure::Lua(cl) = self {
            return &cl.proto.k[id];
        }
        unreachable!()
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
    use crate::luaL;
    #[test]
    /// check if the TValue::Table works
    fn table() {
        let state = luaL::newstate();
        let mut state = state.borrow_mut();
        let t = TValue::new_table();
        state.set_tablev(&t, TValue::new_string("key"), TValue::new_string("value"));
        state.get_tablev(&t, &TValue::new_string("key"), None);
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
        let k = TValue::new_string("key");
        h.insert(k,123);
        let v = h.get(&TValue::new_string("key"));

        assert_eq!(v,Some(&123));
    }
}
