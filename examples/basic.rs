use lualib as lua;
use lualib::luaL;

pub fn main() {
    let mut state = luaL::newstate();
    luaL::open_libs(&mut state).unwrap();
    if let Err(_) = luaL::dostring(
        &mut state,
        "
        function bo(x) return ~x end z=bo(7)
        print(z)
    ",
    ) {
        let msg = lua::to_string(&mut state, -1).unwrap();
        _ = writeln!(state.stderr, "{}", msg);
    }
}
