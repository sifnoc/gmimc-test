#[macro_use]
extern crate bencher;

use bencher::Bencher;
use gmimc_rust_test::{field, gmimc};

pub const M: u128 = 340282366920938463463374557953744961537;
pub const G: u128 = 23953097886125630542083529559205016746;

fn hash_performance(bench: &mut Bencher) {
  let f128 = field::Field::new(M, G);

  let v1 = f128.rand();
  let v2 = f128.rand();
  let v3 = f128.rand();
  let v4 = f128.rand();
  
  let g = gmimc::GMiMC_erf {
    field: f128,
    capacity: 5,
    words: 4,
    round: 166,
  };

  bench.iter(|| {
    g.get_hash_output(&[v1, v2, v3, v4]);
  })
}


benchmark_group!(benches, hash_performance);
benchmark_main!(benches);
