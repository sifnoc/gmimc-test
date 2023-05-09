use ff::PrimeField;

use rand::prelude::random;

use gmimc_rust_test::constants::ARK;

#[derive(PrimeField)]
#[PrimeFieldModulus = "65537"]
#[PrimeFieldGenerator = "7"]
#[PrimeFieldReprEndianness = "little"]
struct Fp([u64; 1]);

fn main() {
    // let mut rng = rand::thread_rng();
    let rand = Fp::from_u128(random::<u128>());
    println!("random generated from f128 Field: {:?}", rand);
    println!("first round constant: {}", ARK[0]);
}
