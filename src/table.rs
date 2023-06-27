//! Lua tables (hash)
//!
//! Implementation of tables (aka arrays, objects, or hash tables).
//! Tables keep its elements in two parts: an array part and a hash part.
//! Non-negative integer keys are all candidates to be kept in the array
//! part.

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{object::TValue, LuaNumber, LuaState};

pub type TableRef = Rc<RefCell<Table>>;

#[derive(Clone, Default)]
pub struct Table {
    pub flags: u8,
    pub metatable: Option<TableRef>,
    pub array: Vec<TValue>,
    pub node: HashMap<TValue, TValue>,
}

impl Table {
    pub fn iter(&self) -> std::slice::Iter<'_, TValue> {
        self.array.iter()
    }
    pub fn pairs(&self) -> std::collections::hash_map::Iter<TValue, TValue> {
        self.node.iter()
    }
    pub fn new() -> Self {
        Self {
            flags: !0,
            metatable: None,
            array: Vec::new(),
            node: HashMap::new(),
        }
    }

    /// Try to find a boundary in table `t'. A `boundary' is an integer index
    /// such that t[i] is non-nil and t[i+1] is nil (and 0 if t[1] is nil).
    pub fn len(&self) -> usize {
        let mut j = self.array.len();
        if j > 0 && self.array[j - 1].is_nil() {
            // there is a boundary in the array part: (binary) search for it
            let mut i = 0;
            while j - i > 1 {
                let m = (i + j) / 2;
                if self.array[m - 1].is_nil() {
                    j = m;
                } else {
                    i = m;
                }
            }
            return i;
        } else if self.node.is_empty() {
            return j;
        }
        // else must find a boundary in hash part
        self.node.len()
    }
    pub fn set(&mut self, key: TValue, value: TValue) {
        match key {
            TValue::Number(n) if n >= 1.0 => {
                let n = n as usize;
                if n > self.array.len() {
                    let new_size = n.max(self.array.len() * 2);
                    self.array.resize(new_size, TValue::Nil);
                }
                self.array[n - 1] = value;
            }
            _ => {
                self.node.insert(key, value);
            }
        }
    }
    pub fn get_num(&mut self, key: usize) -> &TValue {
        if key > self.array.len() {
            &TValue::Nil
        } else {
            &self.array[key - 1]
        }
    }
    /// iterator over both the array and hashmap
    /// returns (next_key, value)
    /// start with key = TValue::Nil then call until it returns (nil,nil)
    pub fn next(&self, key: &TValue) -> (TValue, TValue) {
        match key {
            TValue::Nil => {
                if !self.array.is_empty() {
                    // return first non nil value
                    for (i, v) in self.array.iter().enumerate() {
                        if !v.is_nil() {
                            return (TValue::Number(i as LuaNumber), v.clone());
                        }
                    }
                }
                // return first entry from hashmap or (nil,nil) if empty
                return self
                    .node
                    .iter()
                    .next()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .unwrap_or((TValue::Nil, TValue::Nil));
            }
            TValue::Number(idx) => {
                if idx.fract() == 0.0 && *idx >= 0.0 {
                    if (*idx as usize) < self.array.len()-1 {
                        // key is an integer. get next non nil value
                        for (i, v) in self.array.iter().enumerate().skip(*idx as usize + 1) {
                            if !v.is_nil() {
                                return (TValue::Number(i as LuaNumber), v.clone());
                            }
                        }
                    } else {
                        return self
                            .node
                            .iter()
                            .next()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .unwrap_or((TValue::Nil, TValue::Nil));
                    }
                }
                // key is not an array idx
                let mut found = false;
                for k in self.node.keys() {
                    if k == key {
                        found = true;
                    } else if found {
                        return (k.clone(), self.node.get(k).unwrap().clone());
                    }
                }
                return (TValue::Nil, TValue::Nil);
            }
            _ => {
                let mut found = false;
                for k in self.node.keys() {
                    if k == key {
                        found = true;
                    } else if found {
                        return (k.clone(), self.node.get(k).unwrap().clone());
                    }
                }
                return (TValue::Nil, TValue::Nil);
            }
        }
    }
    pub fn get(&mut self, key: &TValue) -> Option<&TValue> {
        match *key {
            TValue::Nil => Some(&TValue::Nil),
            TValue::Number(n) if n >= 1.0 => {
                let n = n as usize;
                if n > self.array.len() {
                    self.array.resize(n, TValue::Nil);
                }
                Some(&self.array[n - 1])
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
        let mut t = luaH::Table::new();
        let key = TValue::Number(1.0);
        t.set(key.clone(), TValue::from("test1"));
        let v = t.get(&key).unwrap();
        match v {
            TValue::String(r) => {
                assert_eq!(r.as_ref(), "test1");
            }
            _ => {
                assert!(false);
            }
        }
    }
    #[test]
    fn table() {
        let mut t = luaH::Table::new();
        let key = TValue::from("key1");
        t.set(key.clone(), TValue::from("test1"));
        let v = t.get(&key).unwrap();
        match v {
            TValue::String(r) => {
                assert_eq!(r.as_ref(), "test1");
            }
            _ => {
                assert!(false);
            }
        }
    }
}
