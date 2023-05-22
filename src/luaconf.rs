//! Configuration file for Lua

/// LUAI_MAXCALLS limits the number of nested calls.
/// CHANGE it if you need really deep recursive calls. This limit is
/// arbitrary; its only purpose is to stop infinite recursion before
/// exhausting memory.
//pub const LUAI_MAXCALLS: usize = 20000;

/// LUAI_MAXRCALLS is the maximum depth for nested Rust calls (short) and
// syntactical nested non-terminals in a program.
pub const LUAI_MAXRCALLS: usize = 200;

/// LUAI_MAXUPVALUES is the maximum number of upvalues per function
/// (must be smaller than 250).
//pub const LUAI_MAXUPVALUES: usize = 60;

/// LUAI_MAXVARS is the maximum number of local variables per function
/// (must be smaller than 250).
pub const LUAI_MAXVARS: usize = 200;
