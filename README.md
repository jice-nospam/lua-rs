# Lua reimplementation in pure, safe Rust

[![Rust](https://github.com/jice-nospam/lua-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/jice-nospam/lua-rs/actions)

## Status of the project

The current goal is to reimplement each version one after another to have all the latest language features.

There are branches for each version from 5.1 to 5.4, but api and auxlib are not complete on these branches, and the implementation has not been much tested beyond the unit tests and the benchmarks ([see benches/lua](benches/lua)). Consider them in alpha stage.

The core of the 5.1 version of the language is implemented in the [v5.1.x](https://github.com/jice-nospam/lua-rs/tree/v5.1.x) branch.

The core of the 5.2 version of the language is implemented in the [v5.2.x](https://github.com/jice-nospam/lua-rs/tree/v5.2.x) branch with `goto` support. The bitwise operations library is not implemented as it's scrapped in 5.3.

The core of the 5.3 version of the language is implemented in the [v5.3.x](https://github.com/jice-nospam/lua-rs/tree/v5.3.x) branch with integer support including bitwise operations. The utf-8 library is not implemented.

Next to be added : metamethods and consts (5.4).

Current status :

### WON'T BE IMPLEMENTED

- no garbage collector needed as everything is handled gracefully by rust's borrow checker
- hence, no weak table concept neither

### PARTIALLY IMPLEMENTED

- standard libraries : [see src/libs/README.md](src/libs/README.md)
- api and auxlib : [see src/README.md](src/README.md)

### NOT IMPLEMENTED

- user data
- hooks
- coroutines
- to-be-closed variables
- binary chunks
- tables can only be indexed with numbers or strings

## features

- `debug_logs` : disassemble code on stdout when running

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

| name         | lua-rs<br/>5.1 | rlua<br/>5.1 | lua-rs<br/>5.2 | lua-rs<br/>5.3 | rlua<br/>5.3 | lua-rs<br/>5.4 | rlua<br/>5.4 |
|--------------|----------------|--------------|----------------|----------------|--------------|----------------|--------------|
| spectral     |      82.8      |     20.3     |     94.7       |     96.7       |    27.8      |                |              |
| nbody        |       0.2      |      0.1     |      0.2       |      0.6       |     0.5      |                |              |
| mandelbrot   |      27.6      |      6.3     |     31.2       |     34.3       |    11.6      |                |              |
| binary_trees |       1.6      |      0.8     |      1.8       |      2.3       |     1.2      |                |              |

## profiling (linux)

```shell
apt-get install valgrind
valgrind --tool=callgrind target/release/examples/spectral
callgrind_annotate callgrind.out.NNNNN
```
