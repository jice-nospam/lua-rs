//! Tag Methods

use std::rc::Rc;

use crate::{
    debug::{op_int_error, to_int_error},
    opcodes::OpCode,
    state::{CIST_HOOKED, CIST_RUST},
    LuaError, LuaState, TValue,
};

const TM_NAMES: [&str; 25] = [
    "__index",
    "__newindex",
    "__gc",
    "__mode",
    "__len",
    "__eq",
    "__add",
    "__sub",
    "__mul",
    "__mod",
    "__pow",
    "__div",
    "__idiv",
    "__band",
    "__bor",
    "__bxor",
    "__shl",
    "__shr",
    "__unm",
    "__bnot",
    "__lt",
    "__le",
    "__concat",
    "__call",
    "__close",
];

#[derive(PartialEq, Clone, Copy)]
pub enum TagMethod {
    _Index = 0,
    _NewIndex,
    _Gc,
    _Mode,
    _Len,
    _Eq,
    Add,
    Sub,
    Mul,
    Mod,
    Pow,
    Div,
    IntegerDiv,
    BinaryAnd,
    BinaryOr,
    BinaryXor,
    Shl,
    Shr,
    _UnaryMinus,
    BinaryNot,
    _Lt,
    _Le,
    _Concat,
    _Call,
    _Close,
}

impl TryFrom<OpCode> for TagMethod {
    type Error = ();

    fn try_from(value: OpCode) -> Result<Self, Self::Error> {
        match value {
            OpCode::Add => Ok(Self::Add),
            OpCode::Sub => Ok(Self::Sub),
            OpCode::Mul => Ok(Self::Mul),
            OpCode::Mod => Ok(Self::Mod),
            OpCode::Pow => Ok(Self::Pow),
            OpCode::Div => Ok(Self::Div),
            OpCode::IntegerDiv => Ok(Self::IntegerDiv),
            OpCode::BinaryAnd => Ok(Self::BinaryAnd),
            OpCode::BinaryOr => Ok(Self::BinaryOr),
            OpCode::BinaryXor => Ok(Self::BinaryXor),
            OpCode::Shl => Ok(Self::Shl),
            OpCode::Shr => Ok(Self::Shr),
            OpCode::BinaryNot => Ok(Self::BinaryNot),
            _ => Err(()),
        }
    }
}

impl TryFrom<u32> for TagMethod {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TagMethod::_Index),
            6 => Ok(TagMethod::Add),
            7 => Ok(TagMethod::Sub),
            8 => Ok(TagMethod::Mul),
            9 => Ok(TagMethod::Mod),
            10 => Ok(TagMethod::Pow),
            11 => Ok(TagMethod::Div),
            12 => Ok(TagMethod::IntegerDiv),
            13 => Ok(TagMethod::BinaryAnd),
            14 => Ok(TagMethod::BinaryOr),
            15 => Ok(TagMethod::BinaryXor),
            16 => Ok(TagMethod::Shl),
            17 => Ok(TagMethod::Shr),
            18 => Ok(TagMethod::_UnaryMinus),
            19 => Ok(TagMethod::BinaryNot),
            20 => Ok(TagMethod::_Lt),
            21 => Ok(TagMethod::_Le),
            22 => Ok(TagMethod::_Concat),
            23 => Ok(TagMethod::_Call),
            24 => Ok(TagMethod::_Close),
            _ => Err(()),
        }
    }
}

impl LuaState {
    pub(crate) fn get_tm_by_obj(&mut self, p1: usize, event: u32) -> Option<TValue> {
        let mt = self.get_metatable(p1).or_else(|| {
            let typ = self.stack[p1].get_type_name();
            match self.g.mt.get(typ) {
                Some(Some(mt)) => Some(Rc::clone(mt)),
                _ => None,
            }
        });
        if let Some(mt) = mt {
            mt.borrow_mut()
                .get(&TValue::from(TM_NAMES[event as usize]))
                .cloned()
        } else {
            None
        }
    }

    pub(crate) fn call_tm_res(
        &mut self,
        f: &TValue,
        p1: usize,
        p2: usize,
        res: usize,
    ) -> Result<(), LuaError> {
        let func = self.stack.len() - 1;
        self.stack.push(f.clone()); // push function
        self.stack.push(self.stack[p1].clone()); // 1st argument
        self.stack.push(self.stack[p2].clone()); // 2nd argument

        // metamethod may yield only when called from Lua code
        if self.base_ci[self.ci].call_status & (CIST_RUST | CIST_HOOKED) != 0 {
            self.dcall(func, 1)?;
        } else {
            self.dcall_no_yield(func, 1)?;
        }
        self.set_stack_from_idx(res, self.stack.len() - 1); // move result to its place
        self.stack.pop();
        Ok(())
    }
    pub(crate) fn call_bin_tm(
        &mut self,
        p1: usize,
        p2: usize,
        res: usize,
        event: u32,
    ) -> Result<bool, LuaError> {
        // try first operand
        match self.get_tm_by_obj(p1, event) {
            Some(tm) => self.call_tm_res(&tm, p1, p2, res)?,
            None => match self.get_tm_by_obj(p2, event) {
                Some(tm) => self.call_tm_res(&tm, p1, p2, res)?,
                None => return Ok(false),
            },
        }
        Ok(true)
    }

    pub(crate) fn try_bin_tm(
        &mut self,
        p1: usize,
        p2: usize,
        res: usize,
        event: u32,
    ) -> Result<(), LuaError> {
        if !self.call_bin_tm(p1, p2, res, event)? {
            match event.try_into() {
                Ok(TagMethod::BinaryAnd)
                | Ok(TagMethod::BinaryOr)
                | Ok(TagMethod::BinaryXor)
                | Ok(TagMethod::Shl)
                | Ok(TagMethod::Shr)
                | Ok(TagMethod::BinaryNot) => {
                    if self.stack[p1].is_number() && self.stack[p2].is_number() {
                        to_int_error(self, p1, p2)?;
                    } else {
                        op_int_error(self, p1, p2, "perform bitwise operation on")?;
                    }
                }
                Ok(_) => {
                    // calls never return, but to avoid warnings: *//* FALLTHROUGH
                    op_int_error(self, p1, p2, "perform arithmetic on")?;
                }
                Err(_) => unreachable!(),
            }
        }
        Ok(())
    }
}
