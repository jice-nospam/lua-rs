//! Limits, basic types, and some other `installation-dependent' definitions

pub type Instruction=u32;
/// Instruction offset (=program counter)
pub type InstId = usize;

/// maximum stack for a Lua function
pub const MAX_LUA_STACK: usize = 250;

/// maximum value of an int (-2 for safety)
pub const MAX_INT: usize = std::i32::MAX as usize - 2;

/// maximum number of upvalues in a closure (both C and Lua). (Value
///  must fit in an unsigned char.)
pub const MAX_UPVAL: usize = std::u8::MAX as usize;
