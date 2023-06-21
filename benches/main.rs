mod nbody;
mod spectral;
mod mandelbrot;
mod binary_trees;

#[macro_use]
extern crate criterion;


use nbody::{nbody_luars, nbody_rlua};
use spectral::{spectral_luars, spectral_rlua};
use mandelbrot::{mandelbrot_luars,mandelbrot_rlua};
use binary_trees::{binary_trees_luars,binary_trees_rlua};

criterion_group!(
    benches,
    spectral_luars,
    spectral_rlua,
    nbody_luars,
    nbody_rlua,
    mandelbrot_luars,
    mandelbrot_rlua,
    binary_trees_luars,
    binary_trees_rlua,
);
criterion_main!(benches);
