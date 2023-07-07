extern crate lualib;
pub use lualib::api as lua;
pub use lualib::luaL;
#[cfg(target_arch = "wasm32")]
pub use lualib::wasm::js_console;
pub use lualib::LUA_VERSION;

pub fn main() {
    let mut s = luaL::newstate();
    luaL::open_libs(&mut s).unwrap();
    if luaL::dostring(&mut s, "print(_VERSION)").is_err() {
        let msg = lua::to_string(&mut s, -1).unwrap();
        _ = writeln!(s.stderr, "{}", msg);
    }
}
