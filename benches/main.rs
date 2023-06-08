mod nbody;
mod spectral;

#[macro_use]
extern crate criterion;


use nbody::{bench_nbody_luars, bench_nbody_rlua};
use spectral::{bench_spectral_luars, bench_spectral_rlua};

criterion_group!(
    benches,
    bench_spectral_luars,
    bench_spectral_rlua,
    bench_nbody_luars,
    bench_nbody_rlua
);
criterion_main!(benches);
