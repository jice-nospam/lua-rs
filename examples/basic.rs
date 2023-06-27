use lua_rs::luaL;

pub fn main() {
    let mut state = luaL::newstate();
    luaL::open_libs(&mut state).unwrap();
    match luaL::dostring(&mut state, "
    t={1,3,6}
            t.a=9
            z=0
            for k,v in pairs(t) do
                z = z + v
            end
    print(z)
    "){
        Ok(_) => (),
        Err(_) => {
            println!("Error : {}",state.stack.last().unwrap());
        }
    }
}