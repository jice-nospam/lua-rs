pub mod api;
pub mod auxlib;
mod code;
mod func;
mod ldo;
mod lex;
mod libs;
mod limits;
mod luaconf;
mod object;
mod opcodes;
mod parser;
pub mod state;
mod string;
mod table;
mod undump;
mod vm;
mod zio;

pub use api::*;
pub use auxlib as luaL;
pub(crate) use code as luaK;
pub(crate) use func as luaF;
pub(crate) use ldo as luaD;
pub(crate) use parser as luaY;
use state::{LuaStateRef, LuaState};
pub(crate) use table as luaH;
pub(crate) use undump as luaU;
pub(crate) use zio as luaZ;

pub type LuaNumber = f64;
pub type LuaInteger = i64;

pub type LuaRustFunction = fn(&mut LuaState) -> i32;

/// lua bytecode dump header
pub const LUA_SIGNATURE: &str = "\x1BLua";
/// option for multiple returns in `lua_pcall' and `lua_call'
pub const LUA_MULTRET: i32 = -1;
/// minimum Lua stack available to a Rust function
pub const LUA_MINSTACK: usize = 20;
// pseudo-indices
pub const LUA_REGISTRYINDEX: isize = -10000;
pub const LUA_ENVIRONINDEX: isize = -10001;
pub const LUA_GLOBALSINDEX: isize = -10002;
pub const LUA_VERSION: &str = "Lua 5.1";

pub type Reader<T> = fn(LuaStateRef, &T, &mut Vec<char>) -> Result<(),()>;

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::luaL;
    #[test]
    fn basic() {
        let state = luaL::newstate();
        luaL::open_libs(Rc::clone(&state)).unwrap();
        luaL::dostring(state, "print('hello world from lua-rs!')").unwrap();
    }
}
