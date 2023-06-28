//! Configuration file for Lua

/// LUAI_MAXCALLS limits the number of nested calls.
/// CHANGE it if you need really deep recursive calls. This limit is
/// arbitrary; its only purpose is to stop infinite recursion before
/// exhausting memory.
//pub const LUAI_MAXCALLS: usize = 20000;
use crate::LuaNumber;

/// LUAI_MAXRCALLS is the maximum depth for nested Rust calls (short) and
// syntactical nested non-terminals in a program.
const LUAI_MCS_AUX: usize = std::i32::MAX as usize / (4 * std::mem::size_of::<LuaNumber>());
pub const LUAI_MAXRCALLS: usize = if LUAI_MCS_AUX > std::i16::MAX as usize {
    std::i16::MAX as usize
} else {
    LUAI_MCS_AUX
};

/// LUAI_MAXUPVALUES is the maximum number of upvalues per function
/// (must be smaller than 250).
//pub const LUAI_MAXUPVALUES: usize = 60;

/// LUAI_MAXVARS is the maximum number of local variables per function
/// (must be smaller than 250).
pub const LUAI_MAXVARS: usize = 200;
