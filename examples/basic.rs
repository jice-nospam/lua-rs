use lualib as lua;
use lualib::luaL;

pub fn main() {
    let mut state = luaL::newstate();
    luaL::open_libs(&mut state).unwrap();
    if luaL::dostring(
        &mut state,
        "
        function sum(...)
                local args={...};
                return args[1]+args[2]+args[3]
            end
            z=sum(3,8,11)
        print(z)
    ",
    )
    .is_err()
    {
        let msg = lua::to_string(&mut state, -1).unwrap();
        _ = writeln!(state.stderr, "{}", msg);
    }
}
