use lua_rs::luaL;

pub fn main() {
    let mut state = luaL::newstate();
    luaL::open_libs(&mut state).unwrap();
    match luaL::dostring(&mut state, "
    a=0
    t={1,3,5,8}
    function iter(t)
        local i=0
        return function()
            i=i+1
            return t[i]
        end
    end
    for i in iter(t) do a=a+i end
    print(a)
    "){
        Ok(_) => (),
        Err(_) => {
            println!("Error : {}",state.stack.last().unwrap());
        }
    }
}