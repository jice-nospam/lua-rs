use lua_rs as lua;
use lua_rs::luaL;

const SPECTRAL_SRC:&str = include_str!("../benches/lua/spectral.lua");

pub fn main() {
    let mut state = luaL::newstate();
    luaL::open_libs(&mut state).unwrap();
    if let Err(_) = luaL::dostring(&mut state, SPECTRAL_SRC) {
        eprintln!("{}",lua::to_string(&mut state,-1).unwrap());
    }
}