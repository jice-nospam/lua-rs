//! Limits, basic types, and some other `installation-dependent' definitions

pub type Instruction=u32;
/// Instruction offset (=program counter)
pub type InstId = usize;

/// maximum stack for a Lua function
pub const MAX_LUA_STACK: usize = 250;
