#[macro_use]
extern crate bencher;
use gmimc_rust_test::gmimc::gmimc_erf;

use bencher::Bencher;
use ff::PrimeField;
use rand::prelude::random;

fn hash_performance(bench: &mut Bencher) {
    #[derive(PrimeField)]
    #[PrimeFieldModulus = "340282366920938463463374557953744961537"]
    #[PrimeFieldGenerator = "23953097886125630542083529559205016746"]
    #[PrimeFieldReprEndianness = "little"]
    struct F([u64; 3]);

    let gmimc = gmimc_erf::<F, 6> {
        capacity: 5,
        words: 4,
        round: 166,
        _field: std::marker::PhantomData::<F>,
    };

    bench.iter(|| {
        gmimc.get_hash_output(&[
            random::<u128>(),
            random::<u128>(),
            random::<u128>(),
            random::<u128>(),
        ]);
    })
}

benchmark_group!(benches, hash_performance);
benchmark_main!(benches);
