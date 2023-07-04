extern crate lualib;
pub use lualib::api as lua;
pub use lualib::luaL;
pub use lualib::LUA_VERSION;
#[cfg(target_arch = "wasm32")]
pub use lualib::wasm::js_console;

pub fn main() {
    let mut s = luaL::newstate();
    luaL::open_libs(&mut s).unwrap();
    if let Err(_) = luaL::dostring(&mut s, "print(_VERSION)") {
        let msg = lua::to_string(&mut s, -1).unwrap();
        _ = writeln!(s.stderr, "{}", msg);
    }
}
