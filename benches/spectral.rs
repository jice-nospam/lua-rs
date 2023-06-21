extern crate lua_rs;
extern crate rlua;

use criterion::Criterion;

use lua_rs::luaL;

macro_rules! bench_name {
    () => ( "spectral" )
}

const BENCH_SRC:&str = include_str!(concat!("lua/",bench_name!(),".lua"));

pub fn spectral_luars(c: &mut Criterion) {
    let mut state = luaL::newstate();
    luaL::open_libs(&mut state).unwrap();
    c.bench_function(concat!(bench_name!()," lua-rs"), |b| {
        b.iter(|| {
            luaL::dostring(&mut state, BENCH_SRC).unwrap();
        })
    });
}

pub fn spectral_rlua(c: &mut Criterion) {
    let lua = rlua::Lua::new();
        c.bench_function(concat!(bench_name!()," rlua"), |b| {
        b.iter(|| {
            lua.context(|ctx| {
                ctx.load(BENCH_SRC).exec().unwrap();
            });
        })
    });
}


