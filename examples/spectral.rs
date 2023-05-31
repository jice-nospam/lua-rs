use std::rc::Rc;

use lua_rs::luaL;

const SPECTRAL_SRC:&str = include_str!("../benches/lua/spectral.lua");

pub fn main() {
    let state = luaL::newstate();
    luaL::open_libs(Rc::clone(&state)).unwrap();
    luaL::dostring(state, SPECTRAL_SRC).unwrap();
}