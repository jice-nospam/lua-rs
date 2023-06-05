
# compile (native, linux)
```bash
apt-get update
apt-get install gcc g++
cargo run --example basic
```

# run benchmark
`cargo bench`

## bench results
- spectral
    - rlua : 41ms
    - lua-rs : 105ms

# profiling (linux)
```bash
apt-get install valgrind
valgrind --tool=callgrind target/release/examples/spectral
callgrind_annotate callgrind.out.NNNNN
```

```
637,689,714 (71.39%)  ???:_ZN6lua_rs2vm41_$LT$impl$u20$lua_rs..state..LuaState$GT$8vexecute17h03d627370ca21e70E.llvm.5050467725514807213 [/home/jice/lua-rs/target/release/examples/spectral]
 82,029,863 ( 9.18%)  ???:alloc::vec::Vec<T,A>::resize [/home/jice/lua-rs/target/release/examples/spectral]
 44,403,852 ( 4.97%)  ???:lua_rs::state::LuaState::poscall [/home/jice/lua-rs/target/release/examples/spectral]
 38,006,426 ( 4.25%)  ???:_ZN6lua_rs3ldo41_$LT$impl$u20$lua_rs..state..LuaState$GT$8dprecall17h13c97c2c174b3fc1E.llvm.5050467725514807213 [/home/jice/lua-rs/target/release/examples/spectral]
 29,211,939 ( 3.27%)  ???:_ZN4core3ptr53drop_in_place$LT$$u5b$lua_rs..object..TValue$u5d$$GT$17hc110077666e20726E.llvm.7176328389725358494 [/home/jice/lua-rs/target/release/examples/spectral]
 26,413,888 ( 2.96%)  ???:lua_rs::state::LuaState::get_tablev [/home/jice/lua-rs/target/release/examples/spectral]
 20,011,514 ( 2.24%)  ???:lua_rs::table::Table::get [/home/jice/lua-rs/target/release/examples/spectral]
 14,002,206 ( 1.57%)  ???:lua_rs::state::LuaState::close_func [/home/jice/lua-rs/target/release/examples/spectral]
 ```

