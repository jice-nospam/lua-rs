use lualib as lua;
use lualib::luaL;

pub fn main() {
    let mut state = luaL::newstate();
    luaL::open_libs(&mut state).unwrap();
    if luaL::dostring(
        &mut state,
        "
        t=1
        a='4'
        z=3+t+a+'7'
        print(z)
    ",
    )
    .is_err()
    {
        let msg = lua::to_string(&mut state, -1).unwrap();
        _ = writeln!(state.stderr, "{}", msg);
    }
}
