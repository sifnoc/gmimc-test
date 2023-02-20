#[macro_use]
extern crate bencher;

use bencher::Bencher;

use gmimc_rust_test::{field};

pub const M: u128 = 340282366920938463463374557953744961537;
pub const G: u128 = 23953097886125630542083529559205016746;


// 37 ns/iter (+/- 0)
fn exp_3(bench: &mut Bencher) {
  let f128 = field::Field::new(M, G);
  bench.iter(|| {
    let v = f128.rand();
    f128.exp(v, 3u128);  
  })
}

// 25 ns/iter (+/- 0)
fn exp_cube(bench: &mut Bencher) {
  let f128 = field::Field::new(M, G);
  bench.iter(|| {
    let v = f128.rand();
    f128.exp_cube(v);  
  })
}

benchmark_group!(benches, exp_3, exp_cube);
benchmark_main!(benches);
