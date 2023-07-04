use lualib as lua;
use lualib::luaL;

pub fn main() {
    let mut state = luaL::newstate();
    luaL::open_libs(&mut state).unwrap();
    if let Err(_) = luaL::dostring(
        &mut state,
        "
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
    ",
    ) {
        let msg = lua::to_string(&mut state, -1).unwrap();
        _ = writeln!(state.stderr, "{}", msg);
    }
}
