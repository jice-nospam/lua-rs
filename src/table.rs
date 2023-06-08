//! Lua tables (hash)
//!
//! Implementation of tables (aka arrays, objects, or hash tables).
//! Tables keep its elements in two parts: an array part and a hash part.
//! Non-negative integer keys are all candidates to be kept in the array
//! part.

use std::{collections::HashMap, cell::RefCell, rc::Rc};

use crate::object::TValue;

pub type TableRef = Rc<RefCell<Table>>;

#[derive(Clone, Default)]
pub struct Table {
    pub flags: u8,
    pub metatable: Option<TableRef>,
    pub array: Vec<TValue>,
    pub node: HashMap<TValue, TValue>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            flags: !0,
            metatable: None,
            array: Vec::new(),
            node: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: TValue, value: TValue) {
        match key {
            TValue::Number(n) if n >= 1.0 => {
                let n = n as usize;
                if n > self.array.len() {
                    self.array.resize(n, TValue::Nil);
                }
                self.array[n-1] = value;
            }
            _ => {
                self.node.insert(key, value);
            }
        }
    }
    pub fn get(&mut self, key: &TValue) -> Option<&TValue> {
        match *key {
            TValue::Nil => return Some(&TValue::Nil),
            TValue::Number(n) if n >= 1.0 => {
                let n = n as usize;
                if n > self.array.len() {
                    self.array.resize(n, TValue::Nil);
                }
                Some(&self.array[n-1])
            }
            TValue::String(_) => self.node.get(key),
            _ => todo!(),
        }
    }
}



#[cfg(test)]
mod tests {
    use crate::{luaH, object::TValue};
    #[test]
    fn array() {
        let mut t=luaH::Table::new();
        let key = TValue::Number(1.0);
        t.set(key.clone(),TValue::from("test1"));
        let v = t.get(&key).unwrap();
        match v {
            TValue::String(r) => {
                assert_eq!(r.as_ref(),"test1");
            }
            _ => {
                assert!(false);
            }
        }
    }
    #[test]
    fn table() {
        let mut t=luaH::Table::new();
        let key = TValue::from("key1");
        t.set(key.clone(),TValue::from("test1"));
        let v = t.get(&key).unwrap();
        match v {
            TValue::String(r) => {
                assert_eq!(r.as_ref(),"test1");
            }
            _ => {
                assert!(false);
            }
        }
    }
}