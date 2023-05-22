use std::rc::Rc;

use lua_rs::luaL;

pub fn main() {
    let state = luaL::newstate();
    luaL::open_libs(Rc::clone(&state)).unwrap();
    luaL::dostring(state, "print('hello world from lua-rs!')").unwrap();
}