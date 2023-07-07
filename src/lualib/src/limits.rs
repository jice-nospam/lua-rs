//! Limits, basic types, and some other `installation-dependent' definitions

pub type Instruction = u32;
/// Instruction offset (=program counter)
pub type InstId = usize;

/// maximum number of upvalues in a closure (both C and Lua). (Value
///  must fit in an unsigned char.)
pub const MAX_UPVAL: usize = std::u8::MAX as usize;
