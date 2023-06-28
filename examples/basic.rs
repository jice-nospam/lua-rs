use lua_rs::luaL;

pub fn main() {
    let mut state = luaL::newstate();
    luaL::open_libs(&mut state).unwrap();
    match luaL::dostring(&mut state, "
    print(-nil)
    "){
        Ok(_) => (),
        Err(_) => {
            println!("Error : {}",state.stack.last().unwrap());
        }
    }
}