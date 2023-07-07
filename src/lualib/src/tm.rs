//! Tag Methods

use crate::opcodes::OpCode;

#[derive(PartialEq, Clone, Copy)]
pub enum TagMethod {
    _Index = 0,
    _NewIndex,
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
    _BinaryNot,
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
            _ => Err(()),
        }
    }
}
