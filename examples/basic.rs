use lua_rs::luaL;

pub fn main() {
    let mut state = luaL::newstate();
    luaL::open_libs(&mut state).unwrap();
    match luaL::dostring(&mut state, "
    local function fibo(n)
                if n<=2 then
                    return n
                else
                    return fibo(n-1)+fibo(n-2)
                end
            end

            z=fibo(5)
            print(z)
    "){
        Ok(_) => (),
        Err(_) => {
            println!("Error : {}",state.stack.last().unwrap());
        }
    }
}