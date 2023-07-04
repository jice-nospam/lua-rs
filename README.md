# Lua reimplementation in pure, safe Rust

## Status of the project

The current goal is to reimplement each version one after another to have all the latest language features.

The core of the 5.1 version of the language is implemented in the [v5.1.x](https://github.com/jice-nospam/lua-rs/tree/v5.1.x) branch but api and auxlib are very incomplete.
It has not been much tested beyond the unit tests and the benchmarks ([see benches/lua](benches/lua))

Currently working on 5.2 to add goto support.

Next to be added : integers (5.3) and consts (5.4).

I don't plan to implement all the features of the interpreter though. For example, not sure I'll do metamethods.

The long term goal once I have a stable MVP is to evolve this into a different language to remove Lua's part I don't like but if you want to stick with 100% Lua compatibility, this project can be forked.

Current status :

### WON'T BE IMPLEMENTED

- no garbage collector needed as everything is handled gracefully by rust's borrow checker
- hence, no weak table concept neither

### PARTIALLY IMPLEMENTED

- standard libraries : [see src/libs/README.md](src/libs/README.md)
- api and auxlib : [see src/README.md](src/README.md)

### NOT YET IMPLEMENTED

- user data
- metamethods
- hooks
- coroutines
- tables can only be indexed with numbers or strings

## compile (native, linux)

```shell
apt-get update
apt-get install gcc g++
cargo run --example basic
```

## compile (wasm)

Install wasm32 target :

```shell
rustup target install wasm32-unknown-unknown
```

Install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
and [npm](https://nodejs.org/en/download)

Compile with

```shell
wasm-pack build wasm [--features debug_logs]
```

This creates a wasm package in wasm/pkg

Run the demo with

```shell
cd www
npm install
export NODE_OPTIONS=--openssl-legacy-provider
npm run start
```

Application is available at <http://localhost:8080>

## run unit tests

Unit tests are located on the lualib inner crate so you should run them with :

```shell
cargo test -p lualib
```

## run benchmark

Be sure to set the rlua version corresponding to the one you're testing in `Cargo.toml` :

```toml
rlua={version="*",default-features = false, features=["builtin-lua53"]}
```

(note that there is no `builtin-lua52` feature in rlua)

`cargo bench > /dev/null`

## benchmark results

### v5.1.x branch

| name | lua-rs | rlua | ratio |
|------|--------|------|-------|
| spectral | 96ms | 31ms | <span style="color:red">3.41</span> |
| nbody | 2.1ms | 1.9ms | <span style="color:orange">1.1</span> |
| mandelbrot | 36ms | 17.5ms | <span style="color:red">2.1</span> |
| binary_trees | 5.4ms | 3.6ms | <span style="color:orange">1.5</span> |

### v5.2.x branch

| name | lua-rs | rlua | ratio |
|------|--------|------|-------|
| spectral |  |  |  |
| nbody |  |  |  |
| mandelbrot |  |  |  |
| binary_trees |  |  |  |

## profiling (linux)

```shell
apt-get install valgrind
valgrind --tool=callgrind target/release/examples/spectral
callgrind_annotate callgrind.out.NNNNN
```
