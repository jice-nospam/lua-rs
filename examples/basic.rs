use lua_rs::luaL;

pub fn main() {
    let mut state = luaL::newstate();
    luaL::open_libs(&mut state).unwrap();
    match luaL::dostring(&mut state, "
    function facto(n,acc)
                if n==0 then
                    return acc
                else
                    return facto(n-1,acc*n);
                end
            end
            z=facto(7,1)
    print(z)
    "){
        Ok(_) => (),
        Err(_) => {
            println!("Error : {}",state.stack.last().unwrap());
        }
    }
}