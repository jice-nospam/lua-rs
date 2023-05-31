use lua_rs::luaL;

pub fn main() {
    let mut state = luaL::newstate();
    luaL::open_libs(&mut state).unwrap();
    luaL::dostring(&mut state, "print('hello')").unwrap();
}