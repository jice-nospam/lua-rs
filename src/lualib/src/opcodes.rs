//!  Opcodes for Lua virtual machine

use crate::limits::Instruction;

/// We assume that instructions are unsigned 32-bit integers.
/// All instructions have an opcode in the first 7 bits.
/// Instructions can have the following formats:

///         3 3 2 2 2 2 2 2 2 2 2 2 1 1 1 1 1 1 1 1 1 1 0 0 0 0 0 0 0 0 0 0
///         1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0
/// iABC          C(8)     |      B(8)     |k|     A(8)      |   Op(7)     |
/// iABx                Bx(17)               |     A(8)      |   Op(7)     |
/// iAsBx              sBx (signed)(17)      |     A(8)      |   Op(7)     |
/// iAx                           Ax(25)                     |   Op(7)     |
/// isJ                           sJ (signed)(25)            |   Op(7)     |

/// A signed argument is represented in excess K: the represented value is
/// the written unsigned value minus K, where K is half the maximum for the
/// corresponding unsigned argument.

/// size and position of opcode arguments.
pub const SIZE_C: usize = 8;
pub const SIZE_B: usize = 8;
pub const SIZE_BX: usize = SIZE_C + SIZE_B + 1;
pub const SIZE_A: usize = 8;
pub const SIZE_AX: usize = SIZE_BX + SIZE_A;
pub const SIZE_SJ: usize = SIZE_BX + SIZE_A;
pub const SIZE_OP: usize = 7;

pub const POS_OP: usize = 0;
pub const POS_A: usize = POS_OP + SIZE_OP;
pub const POS_K: usize = POS_A + SIZE_A;
pub const POS_B: usize = POS_K + 1;
pub const POS_C: usize = POS_B + SIZE_B;
pub const POS_AX: usize = POS_A;
pub const POS_BX: usize = POS_K;
pub const POS_SJ: usize = POS_A;

/// this bit 1 means constant (0 means register)
pub const BIT_RK: u32 = 1 << POS_K;
pub const MAX_INDEX_RK: usize = BIT_RK as usize - 1;

/// number of list items to accumulate before a SETLIST instruction
pub const LFIELDS_PER_FLUSH: u32 = 50;

#[inline]
pub(crate) const fn rk_is_k(val: u32) -> bool {
    val & BIT_RK != 0
}

pub const MAXARG_A: usize = (1 << SIZE_A) - 1;
pub const MAXARG_B: usize = (1 << SIZE_B) - 1;
pub const MAXARG_C: usize = (1 << SIZE_C) - 1;
pub const MAXARG_AX: usize = (1 << SIZE_AX) - 1;
pub const MAXARG_BX: usize = (1 << SIZE_BX) - 1;
pub const MAXARG_SJ: usize = (1 << SIZE_SJ) - 1;
pub const OFFSET_SBX: i32 = (MAXARG_BX >> 1) as i32;
pub const OFFSET_SJ: i32 = (MAXARG_SJ >> 1) as i32;
pub const OFFSET_SC: i32 = (MAXARG_C >> 1) as i32;
/// value for an invalid register
pub const NO_REG: u32 = MAXARG_A as u32;
pub const NO_JUMP: i32 = -1;

#[cfg(feature = "debug_logs")]
pub(crate) const OPCODE_NAME: [&str; 83] = [
    "move",
    "loadi",
    "loadf",
    "loadk",
    "loadkx",
    "loadfalse",
    "loadfalseskip",
    "loadtrue",
    "loadnil",
    "getupval",
    "setupval",
    "gettabup",
    "gettable",
    "geti",
    "getfield",
    "settabup",
    "settable",
    "seti",
    "setfield",
    "newtable",
    "self",
    "addi",
    "addk",
    "subk",
    "mulk",
    "modk",
    "powk",
    "divk",
    "idivk",
    "bandk",
    "bork",
    "bxork",
    "shri",
    "shli",
    "add",
    "sub",
    "mul",
    "mod",
    "pow",
    "div",
    "idiv",
    "band",
    "bor",
    "bxor",
    "shl",
    "shr",
    "mmbin",
    "mmbini",
    "mmbink",
    "unm",
    "bnot",
    "not",
    "len",
    "concat",
    "close",
    "tbc",
    "jmp",
    "eq",
    "lt",
    "le",
    "eqk",
    "eqi",
    "lti",
    "lei",
    "gti",
    "gei",
    "test",
    "testset",
    "call",
    "tailcall",
    "return",
    "return0",
    "return1",
    "forloop",
    "forprep",
    "tforprep",
    "tforcall",
    "tforloop",
    "setlist",
    "closure",
    "vararg",
    "varargprep",
    "extraarg",
];

#[rustfmt::skip]
mod unformatted {

//                              <--C---><--B--->k<---A--> opcode
pub const MASK_SET_OP: u32 =  0b00000000000000000000000001111111;
pub const MASK_UNSET_OP: u32 =0b11111111111111111111111110000000;
pub const MASK_SET_A: u32 =   0b00000000000000000111111110000000;
pub const MASK_UNSET_A: u32 = 0b11111111111111111000000001111111;
pub const MASK_SET_K: u32 =   0b00000000000000001000000000000000;
pub const MASK_UNSET_K: u32 = 0b11111111111111110111111111111111;
pub const MASK_SET_B: u32 =   0b00000000111111110000000000000000;
pub const MASK_UNSET_B: u32 = 0b11111111000000001111111111111111;
pub const MASK_SET_C: u32 =   0b11111111000000000000000000000000;
pub const MASK_UNSET_C: u32 = 0b00000000111111111111111111111111;
pub const MASK_SET_BX: u32 =  0b11111111111111111000000000000000;
pub const MASK_UNSET_BX: u32 =0b00000000000000000111111111111111;
pub const MASK_SET_AX: u32 =  0b11111111111111111111111110000000;
pub const MASK_SET_SJ: u32 =  0b11111111111111111111111110000000;
pub const _MASK_UNSET_AX: u32=0b00000000000000000000000001111111;
pub const MASK_UNSET_SJ: u32 =0b00000000000000000000000001111111;

#[derive(PartialEq,Clone,Copy)]
#[allow(clippy::tabs_in_doc_comments)]
pub enum OpCode {
    //----------------------------------------------------------------------
    //          args        description
    //name
    //----------------------------------------------------------------------
    ///         A B         R(A) := R(B)
    Move = 0,
    ///         A sBx	    R[A] := sBx	
    LoadI,
    ///         A sBx	    R[A] := (lua_Number)sBx
    LoadF,
    ///         A Bx        R(A) := Kst(Bx)
    LoadK,
    ///         A           R(A) := Kst(extra arg)
    LoadKx,
    ///         A           R(A) := false
    LoadFalse,
    ///         A           R(A) := false, pc ++
    LoadFalseSkip,
    ///         A           R(A) := true
    LoadTrue,
    ///         A B         R[A], R[A+1], ..., R[A+B] := nil
    LoadNil,
    ///         A B         R(A) := UpValue[B]
    GetUpVal,
    ///         A B         UpValue[B] := R(A)
    SetupVal,

    ///         A B C       R[A] := UpValue[B][K[C]:string]	
    GetTabUp,
    ///         A B C       R(A) := R(B)[R(C)]
    GetTable,
    ///         A B C	    R[A] := R[B][C]
    GetI,
    ///         A B C	    R[A] := R[B][K[C]:string]
    GetField,

    ///         A B C       UpValue[A][K[B]:string] := RK(C)
    SetTabUp,
    ///         A B C       R[A][R[B]] := RK(C)	
    SetTable,
    ///         A B C	    R[A][B] := RK(C)
    SetI,
    ///         A B C	    R[A][K[B]:string] := RK(C)
    SetField,

    ///         A B C k     R[A] := {}	
    NewTable,

    ///         A B C       R[A+1] := R[B]; R[A] := R[B][RK(C):string]
    OpSelf,

    ///         A B sC	    R[A] := R[B] + sC	
    AddI,

    ///         A B C	    R[A] := R[B] + K[C]:number
    AddK,    
    ///         A B C	    R[A] := R[B] - K[C]:number
    SubK,
    ///         A B C	    R[A] := R[B] * K[C]:number
    MulK,
    ///         A B C	    R[A] := R[B] % K[C]:number
    ModK,
    ///         A B C	    R[A] := R[B] ^ K[C]:number
    PowK,
    ///         A B C	    R[A] := R[B] / K[C]:number
    DivK,
    ///         A B C	    R[A] := R[B] // K[C]:number
    IntegerDivK,

    ///         A B C	    R[A] := R[B] & K[C]:integer
    BinaryAndK,
    ///         A B C	    R[A] := R[B] | K[C]:integer
    BinaryOrK,
    ///         A B C	    R[A] := R[B] ~ K[C]:integer
    BinaryXorK,

    ///         A B sC	    R[A] := R[B] >> sC
    ShrI,
    ///         A B sC	    R[A] := sC << R[B]
    ShlI,

    ///         A B C	    R[A] := R[B] + R[C]	
    Add,
    ///         A B C	    R[A] := R[B] - R[C]	
    Sub,
    ///         A B C	    R[A] := R[B] * R[C]	
    Mul,
    ///         A B C	    R[A] := R[B] % R[C]	
    Mod,
    ///         A B C	    R[A] := R[B] ^ R[C]	
    Pow,
    ///         A B C	    R[A] := R[B] / R[C]	
    Div,
    ///         A B C	    R[A] := R[B] // R[C]	
    IntegerDiv,

    ///         A B C	    R[A] := R[B] & R[C]	
    BinaryAnd,
    ///         A B C	    R[A] := R[B] | R[C]
    BinaryOr,
    ///         A B C	    R[A] := R[B] ~ R[C]	
    BinaryXor,

    ///         A B C	    R[A] := R[B] << R[C]
    Shl,
    ///         A B C	    R[A] := R[B] >> R[C]
    Shr,

    ///         A B C	    call C metamethod over R[A] and R[B]
    MMBin,
    ///         A sB C k	call C metamethod over R[A] and sB
    MMBinI,
    ///         A B C k		call C metamethod over R[A] and K[B]
    MMBinK,

    ///         A B         R(A) := -R(B)
    UnaryMinus,
    ///         A B         R(A) := ~R(B)
    BinaryNot,
    ///         A B         R(A) := not R(B)
    Not,
    ///         A B         R(A) := #R(B) (length operator)
    Len,

    ///         A B	        R[A] := R[A].. ... ..R[A + B - 1]
    Concat,

    ///         A	        close all upvalues >= R[A]
    Close,
    ///         A	        mark variable A "to be closed"
    ToBeClosed,
    ///        	sJ	        pc += sJ
    Jmp,
    ///         A B k	    if ((R[A] == R[B]) ~= k) then pc++
    Eq,
    ///         A B k	    if ((R[A] <  R[B]) ~= k) then pc++
    Lt,
    ///         A B k	    if ((R[A] <= R[B]) ~= k) then pc++
    Le,

    ///         A B k	    if ((R[A] == K[B]) ~= k) then pc++
    EqK,
    ///         A sB k	    if ((R[A] == sB) ~= k) then pc++
    EqI,
    ///         A sB k	    if ((R[A] < sB) ~= k) then pc++
    LtI,
    ///         A sB k	    if ((R[A] <= sB) ~= k) then pc++
    LeI,
    ///         A sB k	    if ((R[A] > sB) ~= k) then pc++
    GtI,
    ///         A sB k	    if ((R[A] >= sB) ~= k) then pc++
    GeI,

    ///         A k	        if (not R[A] == k) then pc++
    Test,
    ///         A B k	    if (not R[B] == k) then pc++ else R[A] := R[B]
    TestSet,

    ///         A B C	    R[A], ... ,R[A+C-2] := R[A](R[A+1], ... ,R[A+B-1])
    Call,
    ///         A B C k	    return R[A](R[A+1], ... ,R[A+B-1])
    TailCall,

    ///         A B C k	    return R[A], ... ,R[A+B-2]
    Return,
    ///                     return
    Return0,
    ///         A	        return R[A]
    Return1,

    ///         A Bx	    update counters; if loop continues then pc-=Bx;
    ForLoop,
    ///         A Bx	    <check values and prepare counters>; if not to run then pc+=Bx+1;
    ForPrep,

    ///         A Bx	    create upvalue for R[A + 3]; pc+=Bx	
    TForPrep,
    ///         A C	        R[A+4], ... ,R[A+3+C] := R[A](R[A+1], R[A+2]);
    TForCall,
    ///         A Bx	    if R[A+2] ~= nil then { R[A]=R[A+2]; pc -= Bx }	
    TForLoop,

    ///         A B C k	    R[A][C+i] := R[A+i], 1 <= i <= B
    SetList,

    ///         A Bx	    R[A] := closure(KPROTO[Bx])	
    Closure,

    ///         A C	        R[A], R[A+1], ..., R[A+C-2] = vararg
    VarArg,
    ///         A	        (adjust vararg parameters)
    VarArgPrep,

    ///         Ax	        extra (larger) argument for previous opcode
    ExtraArg,
}

}
pub use unformatted::*;

impl OpCode {
    /// convert a binary operator opcode into its version with a constant 2nd operand
    pub(crate) fn to_k(&self) -> Self {
        match self {
            OpCode::Add => OpCode::AddK,
            OpCode::Sub => OpCode::SubK,
            OpCode::Mul => OpCode::MulK,
            OpCode::Mod => OpCode::ModK,
            OpCode::Pow => OpCode::AddK,
            OpCode::Div => OpCode::DivK,
            OpCode::IntegerDiv => OpCode::IntegerDivK,
            OpCode::BinaryAnd => OpCode::BinaryAndK,
            OpCode::BinaryOr => OpCode::BinaryOrK,
            OpCode::BinaryXor => OpCode::BinaryXorK,
            OpCode::Shl => OpCode::ShlI,
            OpCode::Shr => OpCode::ShrI,
            _ => unreachable!(),
        }
    }
    pub(crate) fn is_test(&self) -> bool {
        matches!(
            self,
            OpCode::Eq
                | OpCode::Lt
                | OpCode::Le
                | OpCode::EqK
                | OpCode::EqI
                | OpCode::LtI
                | OpCode::LeI
                | OpCode::GtI
                | OpCode::GeI
                | OpCode::Test
                | OpCode::TestSet
        )
    }
    pub(crate) fn is_mm(&self) -> bool {
        matches!(self, OpCode::MMBin | OpCode::MMBinI | OpCode::MMBinK)
    }
    pub(crate) fn sets_a(&self) -> bool {
        !matches!(
            self,
            OpCode::SetupVal
                | OpCode::SetTabUp
                | OpCode::SetTable
                | OpCode::SetI
                | OpCode::SetField
                | OpCode::MMBin
                | OpCode::MMBinI
                | OpCode::MMBinK
                | OpCode::Close
                | OpCode::ToBeClosed
                | OpCode::Jmp
                | OpCode::Eq
                | OpCode::Lt
                | OpCode::Le
                | OpCode::EqI
                | OpCode::EqK
                | OpCode::LtI
                | OpCode::LeI
                | OpCode::GtI
                | OpCode::GeI
                | OpCode::Test
                | OpCode::Return
                | OpCode::Return0
                | OpCode::Return1
                | OpCode::TForPrep
                | OpCode::TForCall
                | OpCode::SetList
                | OpCode::ExtraArg
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
                | OpCode::BinaryNot
                | OpCode::Not
                | OpCode::Len
                | OpCode::Return
                | OpCode::VarArg
        )
    }
    pub(crate) fn is_ac(&self) -> bool {
        matches!(self, OpCode::TForCall | OpCode::VarArg)
    }
    pub(crate) fn is_a(&self) -> bool {
        matches!(self, OpCode::VarArgPrep)
    }
    pub(crate) fn is_abx(&self) -> bool {
        matches!(
            self,
            OpCode::LoadK
                | OpCode::LoadKx
                | OpCode::ForLoop
                | OpCode::ForPrep
                | OpCode::TForPrep
                | OpCode::TForLoop
                | OpCode::Closure
        )
    }
    pub(crate) fn is_asbx(&self) -> bool {
        matches!(self, OpCode::LoadI | OpCode::LoadF)
    }
    #[cfg(feature = "debug_logs")]
    pub(crate) fn is_ax(&self) -> bool {
        matches!(self, OpCode::ExtraArg)
    }
    pub(crate) fn is_ak(&self) -> bool {
        matches!(self, OpCode::Test)
    }
    pub(crate) fn is_absc(&self) -> bool {
        matches!(self, OpCode::AddI | OpCode::ShrI | OpCode::ShlI)
    }
    pub(crate) fn is_abc(&self) -> bool {
        matches!(
            self,
            OpCode::Move
                | OpCode::LoadFalse
                | OpCode::LoadFalseSkip
                | OpCode::LoadTrue
                | OpCode::LoadNil
                | OpCode::GetUpVal
                | OpCode::SetupVal
                | OpCode::GetTabUp
                | OpCode::GetTable
                | OpCode::GetI
                | OpCode::GetField
                | OpCode::SetTabUp
                | OpCode::SetTable
                | OpCode::SetI
                | OpCode::SetField
                | OpCode::NewTable
                | OpCode::OpSelf
                | OpCode::AddK
                | OpCode::SubK
                | OpCode::MulK
                | OpCode::ModK
                | OpCode::PowK
                | OpCode::DivK
                | OpCode::IntegerDivK
                | OpCode::BinaryAndK
                | OpCode::BinaryOrK
                | OpCode::BinaryXorK
                | OpCode::Add
                | OpCode::Sub
                | OpCode::Mul
                | OpCode::Mod
                | OpCode::Pow
                | OpCode::Div
                | OpCode::IntegerDiv
                | OpCode::BinaryAnd
                | OpCode::BinaryOr
                | OpCode::BinaryXor
                | OpCode::Shl
                | OpCode::Shr
                | OpCode::MMBin
                | OpCode::MMBinI
                | OpCode::MMBinK
                | OpCode::UnaryMinus
                | OpCode::BinaryNot
                | OpCode::Not
                | OpCode::Len
                | OpCode::Concat
                | OpCode::Close
                | OpCode::ToBeClosed
                | OpCode::Eq
                | OpCode::Lt
                | OpCode::Le
                | OpCode::EqK
                | OpCode::EqI
                | OpCode::LtI
                | OpCode::LeI
                | OpCode::GtI
                | OpCode::GeI
                | OpCode::TestSet
                | OpCode::Call
                | OpCode::TailCall
                | OpCode::Return
                | OpCode::Return0
                | OpCode::Return1
                | OpCode::TForCall
                | OpCode::SetList
        )
    }
    pub(crate) fn is_sj(&self) -> bool {
        matches!(self, OpCode::Jmp)
    }
}

impl TryFrom<u32> for OpCode {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Move),
            1 => Ok(Self::LoadI),
            2 => Ok(Self::LoadF),
            3 => Ok(Self::LoadK),
            4 => Ok(Self::LoadKx),
            5 => Ok(Self::LoadFalse),
            6 => Ok(Self::LoadFalseSkip),
            7 => Ok(Self::LoadTrue),
            8 => Ok(Self::LoadNil),
            9 => Ok(Self::GetUpVal),
            10 => Ok(Self::SetupVal),

            11 => Ok(Self::GetTabUp),
            12 => Ok(Self::GetTable),
            13 => Ok(Self::GetI),
            14 => Ok(Self::GetField),

            15 => Ok(Self::SetTabUp),
            16 => Ok(Self::SetTable),
            17 => Ok(Self::SetI),
            18 => Ok(Self::SetField),

            19 => Ok(Self::NewTable),

            20 => Ok(Self::OpSelf),

            21 => Ok(Self::AddI),
            22 => Ok(Self::AddK),
            23 => Ok(Self::SubK),
            24 => Ok(Self::MulK),
            25 => Ok(Self::ModK),
            26 => Ok(Self::PowK),
            27 => Ok(Self::DivK),
            28 => Ok(Self::IntegerDivK),

            29 => Ok(Self::BinaryAndK),
            30 => Ok(Self::BinaryOrK),
            31 => Ok(Self::BinaryXorK),

            32 => Ok(Self::ShrI),
            33 => Ok(Self::ShlI),

            34 => Ok(Self::Add),
            35 => Ok(Self::Sub),
            36 => Ok(Self::Mul),
            37 => Ok(Self::Mod),
            38 => Ok(Self::Pow),
            39 => Ok(Self::Div),
            40 => Ok(Self::IntegerDiv),

            41 => Ok(Self::BinaryAnd),
            42 => Ok(Self::BinaryOr),
            43 => Ok(Self::BinaryXor),
            44 => Ok(Self::Shl),
            45 => Ok(Self::Shr),

            46 => Ok(Self::MMBin),
            47 => Ok(Self::MMBinI),
            48 => Ok(Self::MMBinK),

            49 => Ok(Self::UnaryMinus),
            50 => Ok(Self::BinaryNot),
            51 => Ok(Self::Not),
            52 => Ok(Self::Len),

            53 => Ok(Self::Concat),

            54 => Ok(Self::Close),
            55 => Ok(Self::ToBeClosed),
            56 => Ok(Self::Jmp),
            57 => Ok(Self::Eq),
            58 => Ok(Self::Lt),
            59 => Ok(Self::Le),

            60 => Ok(Self::EqK),
            61 => Ok(Self::EqI),
            62 => Ok(Self::LtI),
            63 => Ok(Self::LeI),
            64 => Ok(Self::GtI),
            65 => Ok(Self::GeI),

            66 => Ok(Self::Test),
            67 => Ok(Self::TestSet),

            68 => Ok(Self::Call),
            69 => Ok(Self::TailCall),

            70 => Ok(Self::Return),
            71 => Ok(Self::Return0),
            72 => Ok(Self::Return1),

            73 => Ok(Self::ForLoop),
            74 => Ok(Self::ForPrep),

            75 => Ok(Self::TForPrep),
            76 => Ok(Self::TForCall),
            77 => Ok(Self::TForLoop),

            78 => Ok(Self::SetList),

            79 => Ok(Self::Closure),

            80 => Ok(Self::VarArg),
            81 => Ok(Self::VarArgPrep),

            82 => Ok(Self::ExtraArg),
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
pub(crate) fn get_arg_sb(i: Instruction) -> i32 {
    get_arg_b(i) as i32 - OFFSET_SC
}
#[inline]
pub(crate) fn get_arg_sc(i: Instruction) -> i32 {
    get_arg_c(i) as i32 - OFFSET_SC
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
    (get_arg_bx(i) as i64 - OFFSET_SBX as i64) as i32
}
pub(crate) fn _set_arg_sbx(dest: &mut Instruction, sbx: i32) {
    set_arg_bx(dest, (sbx + OFFSET_SBX) as u32);
}

#[inline]
pub(crate) fn get_arg_k(i: Instruction) -> u32 {
    (i & MASK_SET_K) >> POS_K
}
pub(crate) fn set_arg_k(dest: &mut Instruction, arg: u32) {
    *dest = (*dest & MASK_UNSET_K) | ((arg << POS_K) & MASK_SET_K);
}

#[inline]
pub(crate) fn get_arg_sj(i: Instruction) -> i32 {
    ((i & MASK_SET_SJ) >> POS_SJ) as i32 - OFFSET_SJ
}
pub(crate) fn set_arg_sj(dest: &mut Instruction, sj: i32) {
    *dest = (*dest & MASK_UNSET_SJ) | (((sj + OFFSET_SJ) << POS_SJ) as u32 & MASK_SET_SJ);
}

pub(crate) fn create_abck(opcode: u32, a: i32, b: i32, c: i32, k: u32) -> u32 {
    opcode
        | ((a << POS_A) as u32 & MASK_SET_A)
        | ((b << POS_B) as u32 & MASK_SET_B)
        | ((c << POS_C) as u32 & MASK_SET_C)
        | ((k << POS_K) & MASK_SET_K)
}

pub(crate) fn create_abx(opcode: u32, a: i32, bx: u32) -> u32 {
    opcode | ((a << POS_A) as u32 & MASK_SET_A) | ((bx << POS_BX) & MASK_SET_BX)
}

pub(crate) fn create_ax(opcode: u32, ax: u32) -> u32 {
    opcode | ((ax << POS_AX) & MASK_SET_A)
}

pub(crate) fn create_sj(opcode: u32, j: u32, k: u32) -> u32 {
    opcode | ((j << POS_SJ) & MASK_SET_SJ) | ((k << POS_K) & MASK_SET_K)
}
