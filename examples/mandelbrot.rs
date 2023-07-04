use lualib as lua;
use lualib::luaL;

const LUA_SRC:&str = include_str!("../benches/lua/mandelbrot.lua");

pub fn main() {
    let mut state = luaL::newstate();
    luaL::open_libs(&mut state).unwrap();
    if let Err(_) = luaL::dostring(&mut state, LUA_SRC) {
        eprintln!("{}",lua::to_string(&mut state,-1).unwrap());
    }
}