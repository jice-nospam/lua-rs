extern crate lua_rs;
#[macro_use]
extern crate criterion;
extern crate rlua;

use criterion::Criterion;

use lua_rs::luaL;

const SPECTRAL_SRC:&str = include_str!("lua/spectral.lua");

pub fn bench_spectral_luars(c: &mut Criterion) {
    let mut state = luaL::newstate();
    luaL::open_libs(&mut state).unwrap();
    c.bench_function("spectral lua-rs", |b| {
        b.iter(|| {
            luaL::dostring(&mut state, SPECTRAL_SRC).unwrap();
        })
    });
}

pub fn bench_spectral_rlua(c: &mut Criterion) {
    let lua = rlua::Lua::new();
        c.bench_function("spectral rlua", |b| {
        b.iter(|| {
            lua.context(|ctx| {
                ctx.load(SPECTRAL_SRC).exec().unwrap();
            });
        })
    });
}

criterion_group!(benches, bench_spectral_luars, bench_spectral_rlua);
criterion_main!(benches);
