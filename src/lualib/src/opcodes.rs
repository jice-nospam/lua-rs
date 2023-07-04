//!  Opcodes for Lua virtual machine

use crate::limits::Instruction;

///   We assume that instructions are unsigned numbers.
///   All instructions have an opcode in the first 6 bits.
///   Instructions can have the following fields:
///     `A' : 8 bits
///     `B' : 9 bits
///     `C' : 9 bits
///     `Bx' : 18 bits (`B' and `C' together)
///     `sBx' : signed Bx

///   A signed argument is represented in excess K; that is, the number
///   value is the unsigned value minus K. K is exactly the maximum value
///   for that argument (so that -max is represented by 0, and +max is
///   represented by 2*max), which is half the maximum for the corresponding
///   unsigned argument.

/// size and position of opcode arguments.
pub const SIZE_C: usize = 9;
pub const SIZE_B: usize = 9;
pub const SIZE_BX: usize = SIZE_C + SIZE_B;
pub const SIZE_A: usize = 8;
pub const SIZE_AX: usize = SIZE_C + SIZE_B + SIZE_A;
pub const SIZE_OP: usize = 6;

pub const POS_OP: usize = 0;
pub const POS_A: usize = POS_OP + SIZE_OP;
pub const POS_C: usize = POS_A + SIZE_A;
pub const POS_B: usize = POS_C + SIZE_C;
pub const POS_AX: usize = POS_A;
pub const POS_BX: usize = POS_C;

/// this bit 1 means constant (0 means register)
pub const BIT_RK: u32 = 1 << (SIZE_B - 1);
pub const MAX_INDEX_RK: usize = BIT_RK as usize - 1;

/// number of list items to accumulate before a SETLIST instruction
pub const LFIELDS_PER_FLUSH: u32 = 50;

#[inline]
pub(crate) const fn rk_as_k(val: u32) -> u32 {
    val | BIT_RK
}
#[inline]
pub(crate) const fn rk_is_k(val: u32) -> bool {
    val & BIT_RK != 0
}

pub const MAXARG_A: usize = (1 << SIZE_A) - 1;
pub const _MAXARG_B: usize = (1 << SIZE_B) - 1;
pub const MAXARG_C: usize = (1 << SIZE_C) - 1;
pub const MAXARG_AX: usize = (1 << SIZE_AX) - 1;
pub const MAXARG_BX: usize = (1 << SIZE_BX) - 1;
pub const MAXARG_SBX: i32 = (MAXARG_BX >> 1) as i32;
/// value for an invalid register
pub const NO_REG: u32 = MAXARG_A as u32;
pub const NO_JUMP: i32 = -1;

#[cfg(feature = "debug_logs")]
pub(crate) const OPCODE_NAME: [&str; 40] = [
    "move", "loadk", "loadkx", "loadbool", "loadnil", "getupval", "gettabup", "gettable",
    "settabup", "setupval", "settable", "newtable", "opself", "add", "sub", "mul", "div", "mod",
    "pow", "unm", "not", "len", "concat", "jmp", "eq", "lt", "le", "test", "testset", "call",
    "tailcall", "return", "forloop", "forprep", "tforcall", "tforloop", "setlist", "closure",
    "vararg", "extraarg",
];

#[rustfmt::skip]
mod unformatted {

//                              <---B---><---C---><---A-->opcode
pub const MASK_SET_OP: u32 =  0b00000000000000000000000000111111;
pub const MASK_UNSET_OP: u32 =0b11111111111111111111111111000000;
pub const MASK_SET_A: u32 =   0b00000000000000000011111111000000;
pub const MASK_SET_C: u32 =   0b00000000011111111100000000000000;
pub const MASK_SET_B: u32 =   0b11111111100000000000000000000000;
pub const MASK_UNSET_A: u32 = 0b11111111111111111100000000111111;
pub const MASK_UNSET_C: u32 = 0b11111111100000000011111111111111;
pub const MASK_UNSET_B: u32 = 0b00000000011111111111111111111111;
pub const MASK_SET_BX: u32 =  0b11111111111111111100000000000000;
pub const MASK_UNSET_BX: u32 =0b00000000000000000011111111111111;
pub const MASK_SET_AX: u32 =  0b11111111111111111111111111100000;
pub const _MASK_UNSET_AX: u32=0b00000000000000000000000000011111;

#[derive(PartialEq,Clone,Copy)]
pub enum OpCode {
    //----------------------------------------------------------------------
    //          args    description
    //name
    //----------------------------------------------------------------------
    ///         A B     R(A) := R(B)
    Move = 0,
    ///         A Bx    R(A) := Kst(Bx)
    LoadK,
    ///         A       R(A) := Kst(extra arg)
    LoadKx,
    ///         A B C   R(A) := (Bool)B; if (C) pc++
    LoadBool,
    ///         A B     R(A) := ... := R(B) := nil
    LoadNil,
    ///         A B     R(A) := UpValue[B]
    GetUpVal,
    ///         A B C   R(A) := UpValue[B][RK(C)]
    GetTabUp,
    ///         A B C   R(A) := R(B)[RK(C)]
    GetTable,
    ///         A B C   UpValue[A][RK(B)] := RK(C)
    SetTabUp,
    ///         A B     UpValue[B] := R(A)
    SetupVal,
    ///         A B C   R(A)[RK(B)] := RK(C)
    SetTable,
    ///         A B C   R(A) := {} (size = B,C)
    NewTable,
    ///         A B C   R(A+1) := R(B); R(A) := R(B)[RK(C)]
    OpSelf,
    ///         A B C   R(A) := RK(B) + RK(C)
    Add,
    ///         A B C   R(A) := RK(B) - RK(C)
    Sub,
    ///         A B C   R(A) := RK(B) * RK(C)
    Mul,
    ///         A B C   R(A) := RK(B) / RK(C)
    Div,
    ///         A B C   R(A) := RK(B) % RK(C)
    Mod,
    ///         A B C   R(A) := RK(B) ^ RK(C)
    Pow,
    ///         A B     R(A) := -R(B)
    UnaryMinus,
    ///         A B     R(A) := not R(B)
    Not,
    ///         A B     R(A) := length of R(B)
    Len,
    ///         A B C   R(A) := R(B).. ... ..R(C)
    Concat,
    ///         sBx     pc+=sBx
    Jmp,
    ///         A B C   if ((RK(B) == RK(C)) ~= A) then pc++
    Eq,
    ///         A B C   if ((RK(B) <  RK(C)) ~= A) then pc++
    Lt,
    ///         A B C   if ((RK(B) <= RK(C)) ~= A) then pc++
    Le,
    ///         A C     if not (R(A) <=> C) then pc++
    Test,
    ///         A B C   if (R(B) <=> C) then R(A) := R(B) else pc++
    TestSet,
    ///         A B C   R(A), ... ,R(A+C-2) := R(A)(R(A+1), ... ,R(A+B-1))
    Call,
    ///         A B C   return R(A)(R(A+1), ... ,R(A+B-1))
    TailCall,
    ///         A B     return R(A), ... ,R(A+B-2)    (see note)
    Return,
    ///         A sBx   R(A)+=R(A+2); if R(A) <?= R(A+1) then { pc+=sBx; R(A+3)=R(A) }
    ForLoop,
    ///         A sBx   R(A)-=R(A+2); pc+=sBx
    ForPrep,
    ///         A C     R(A+3), ... ,R(A+2+C) := R(A)(R(A+1), R(A+2));
    TForCall,
    ///         A sBx	if R(A+1) ~= nil then { R(A)=R(A+1); pc += sBx }
    TForLoop,
    ///         A B C   R(A)[(C-1)*FPF+i] := R(A+i), 1 <= i <= B
    SetList,
    ///         A Bx    R(A) := closure(KPROTO[Bx])
    Closure,
    ///         A B     R(A), R(A+1), ..., R(A+B-1) = vararg
    VarArg,
    ///         Ax      extra (larger) argument for previous opcode
    ExtraArg,
}

}
pub use unformatted::*;

impl OpCode {
    pub(crate) fn is_test(&self) -> bool {
        matches!(
            self,
            OpCode::Eq
                | OpCode::Lt
                | OpCode::Le
                | OpCode::Test
                | OpCode::TestSet
                | OpCode::TForLoop
        )
    }
    #[cfg(feature = "debug_logs")]
    pub(crate) fn is_ab(&self) -> bool {
        matches!(
            self,
            OpCode::Move
                | OpCode::LoadNil
                | OpCode::GetUpVal
                | OpCode::SetupVal
                | OpCode::UnaryMinus
                | OpCode::Not
                | OpCode::Len
                | OpCode::Return
                | OpCode::VarArg
        )
    }
    #[cfg(feature = "debug_logs")]
    pub(crate) fn is_ac(&self) -> bool {
        matches!(self, OpCode::TForCall | OpCode::Test)
    }
    #[cfg(feature = "debug_logs")]
    pub(crate) fn is_abx(&self) -> bool {
        matches!(self, OpCode::LoadK | OpCode::LoadKx | OpCode::Closure)
    }
    #[cfg(feature = "debug_logs")]
    pub(crate) fn is_asbx(&self) -> bool {
        matches!(
            self,
            OpCode::Jmp | OpCode::ForLoop | OpCode::ForPrep | OpCode::TForLoop
        )
    }
    #[cfg(feature = "debug_logs")]
    pub(crate) fn is_ax(&self) -> bool {
        matches!(self, OpCode::ExtraArg)
    }
}

impl TryFrom<u32> for OpCode {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Move),
            1 => Ok(Self::LoadK),
            2 => Ok(Self::LoadKx),
            3 => Ok(Self::LoadBool),
            4 => Ok(Self::LoadNil),
            5 => Ok(Self::GetUpVal),
            6 => Ok(Self::GetTabUp),
            7 => Ok(Self::GetTable),
            8 => Ok(Self::SetTabUp),
            9 => Ok(Self::SetupVal),
            10 => Ok(Self::SetTable),
            11 => Ok(Self::NewTable),
            12 => Ok(Self::OpSelf),
            13 => Ok(Self::Add),
            14 => Ok(Self::Sub),
            15 => Ok(Self::Mul),
            16 => Ok(Self::Div),
            17 => Ok(Self::Mod),
            18 => Ok(Self::Pow),
            19 => Ok(Self::UnaryMinus),
            20 => Ok(Self::Not),
            21 => Ok(Self::Len),
            22 => Ok(Self::Concat),
            23 => Ok(Self::Jmp),
            24 => Ok(Self::Eq),
            25 => Ok(Self::Lt),
            26 => Ok(Self::Le),
            27 => Ok(Self::Test),
            28 => Ok(Self::TestSet),
            29 => Ok(Self::Call),
            30 => Ok(Self::TailCall),
            31 => Ok(Self::Return),
            32 => Ok(Self::ForLoop),
            33 => Ok(Self::ForPrep),
            34 => Ok(Self::TForCall),
            35 => Ok(Self::TForLoop),
            36 => Ok(Self::SetList),
            37 => Ok(Self::Closure),
            38 => Ok(Self::VarArg),
            39 => Ok(Self::ExtraArg),
            _ => Err(()),
        }
    }
}

#[inline]
pub(crate) fn get_opcode(i: Instruction) -> OpCode {
    OpCode::try_from(i & MASK_SET_OP).unwrap()
}
pub(crate) fn set_opcode(dest: &mut Instruction, arg: u32) {
    *dest = (*dest & MASK_UNSET_OP) | (arg & MASK_SET_OP);
}

#[inline]
pub(crate) fn get_arg_a(i: Instruction) -> u32 {
    (i & MASK_SET_A) >> POS_A
}
pub(crate) fn set_arg_a(dest: &mut Instruction, arg: u32) {
    *dest = (*dest & MASK_UNSET_A) | ((arg << POS_A) & MASK_SET_A);
}

#[inline]
pub(crate) fn get_arg_b(i: Instruction) -> u32 {
    (i & MASK_SET_B) >> POS_B
}
pub(crate) fn set_arg_b(dest: &mut Instruction, arg: u32) {
    *dest = (*dest & MASK_UNSET_B) | ((arg << POS_B) & MASK_SET_B);
}
#[inline]
#[cfg(feature = "debug_logs")]
pub(crate) fn get_arg_sb(i: Instruction) -> i32 {
    let b = get_arg_b(i);
    if b >= 256 {
        -((b - 256) as i32) - 1
    } else {
        b as i32
    }
}
#[inline]
#[cfg(feature = "debug_logs")]
pub(crate) fn get_arg_sc(i: Instruction) -> i32 {
    let c = get_arg_c(i);
    if c >= 256 {
        -((c - 256) as i32) - 1
    } else {
        c as i32
    }
}

#[inline]
pub(crate) fn get_arg_c(i: Instruction) -> u32 {
    (i & MASK_SET_C) >> POS_C
}
pub(crate) fn set_arg_c(dest: &mut Instruction, arg: u32) {
    *dest = (*dest & MASK_UNSET_C) | ((arg << POS_C) & MASK_SET_C);
}

#[inline]
pub(crate) fn get_arg_bx(i: Instruction) -> u32 {
    (i & MASK_SET_BX) >> POS_BX
}
pub(crate) fn set_arg_bx(dest: &mut Instruction, arg: u32) {
    *dest = (*dest & MASK_UNSET_BX) | ((arg << POS_BX) & MASK_SET_BX);
}

#[inline]
pub(crate) fn get_arg_ax(i: Instruction) -> u32 {
    (i & MASK_SET_AX) >> POS_AX
}

#[inline]
pub(crate) fn get_arg_sbx(i: Instruction) -> i32 {
    (get_arg_bx(i) as i64 - MAXARG_SBX as i64) as i32
}
pub(crate) fn set_arg_sbx(dest: &mut Instruction, sbx: i32) {
    set_arg_bx(dest, (sbx + MAXARG_SBX) as u32);
}

pub(crate) fn create_abc(opcode: u32, a: i32, b: i32, c: i32) -> u32 {
    opcode
        | ((a << POS_A) as u32 & MASK_SET_A)
        | ((b << POS_B) as u32 & MASK_SET_B)
        | ((c << POS_C) as u32 & MASK_SET_C)
}

pub(crate) fn create_abx(opcode: u32, a: i32, bx: u32) -> u32 {
    opcode | ((a << POS_A) as u32 & MASK_SET_A) | ((bx << POS_BX) & MASK_SET_BX)
}

pub(crate) fn create_ax(opcode: u32, ax: u32) -> u32 {
    opcode | ((ax << POS_AX) & MASK_SET_A)
}

pub(crate) fn is_reg_constant(reg: u32) -> bool {
    reg & BIT_RK != 0
}
