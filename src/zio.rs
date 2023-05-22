//! a generic input stream interface

use std::rc::Rc;

use crate::{LuaStateRef, Reader};

pub struct Zio<T> {
    /// bytes still unread
    n: usize,
    /// current position in buffer
    offset: usize,
    /// buffer containing current chunk
    buffer: Vec<char>,
    reader: Reader<T>,
    /// reader additional data
    data: T,
    /// Lua state (for reader)
    state: LuaStateRef,
}

impl<T> Zio<T> {
    pub fn new(state: LuaStateRef, reader: Reader<T>, data: T) -> Self {
        Self {
            n: 0,
            offset: 0,
            reader,
            data,
            state,
            buffer: Vec::new(),
        }
    }
    /// return next character without consuming it or None if EOF
    pub fn look_ahead(&mut self) -> Option<char> {
        if self.n == 0 {
            if self.fill().is_none() {
                return None;
            } else {
                // don't consume first character
                self.offset=0;
                self.n+=1;
            }
        }
        Some(self.buffer[self.offset])
    }
    /// consume and return the next character or None if EOF
    pub fn getc(&mut self) -> Option<char> {
        if self.n == 0 {
            self.fill()
        } else {
            self.n-=1;
            self.offset+=1;
            Some(self.buffer[self.offset-1])
        }
    }
    /// load a new chunk and return the first character or None is EOF
    fn fill(&mut self) -> Option<char> {
        match (self.reader)(Rc::clone(&self.state), &self.data, &mut self.buffer) {
            Ok(_) => {
                self.n = self.buffer.len()-1;
                self.offset = 1;
                Some(self.buffer[0])
            }
            Err(_) => {
                None
            }
        }
    }
}
