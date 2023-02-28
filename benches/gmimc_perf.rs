#[macro_use]
extern crate bencher;

use bencher::Bencher;
use gmimc_rust_test::{field, gmimc, avalanche};

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
    let mut hash_output = [0u8; 32];
    let input_value = avalanche::array_u128_to_u8(&[v1, v2, v3, v4]);
    let values = gmimc::as_bytes(&input_value);
    g.get_hash_output(values, &mut hash_output);
  })
}

benchmark_group!(benches, hash_performance);
benchmark_main!(benches);
