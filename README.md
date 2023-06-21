
# compile (native, linux)
```bash
apt-get update
apt-get install gcc g++
cargo run --example basic
```

# run benchmark
`cargo bench`

## benchmark results

| name | lua-rs | rlua | ratio |
|------|--------|------|-------|
| spectral | 96ms | 31ms | <span style="color:red">3.41</span> |
| nbody | 2.1ms | 1.9ms | <span style="color:orange">1.1</span> |
| mandelbrot | KO | 19.8ms | <span style="color:red">KO</span> |
| binary_trees | 5.4ms | 3.6ms | <span style="color:orange">1.5</span> |

# profiling (linux)
```bash
apt-get install valgrind
valgrind --tool=callgrind target/release/examples/spectral
callgrind_annotate callgrind.out.NNNNN
```

# TODO
- user data
- metamethods
- hooks
