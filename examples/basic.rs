use lua_rs::luaL;

pub fn main() {
    let mut state = luaL::newstate();
    luaL::open_libs(&mut state).unwrap();
    match luaL::dostring(&mut state, "
    local z=0
    for i=0,3 do
        for j=0,3 do
            z=z+i*j
            for k=1,10 do
                if k > 5 then
                    break;
                end
            end
        end
    end
    print(z)
    "){
        Ok(_) => (),
        Err(_) => {
            println!("Error : {}",state.stack.last().unwrap());
        }
    }
}