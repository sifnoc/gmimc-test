// mod constants;
// mod gmimc;

use std::fmt::Write;
use std::{mem, slice};

use gmimc_rust_test::constants::ARK;
use gmimc_rust_test::field::Field;


fn main() {
 let f128 = Field::new(340282366920938463463374557953744961537, 23953097886125630542083529559205016746);
 let rand = f128.rand();
 println!("random generated from f128 Field: {}", rand);
 println!("first round constant: {}", ARK[0]);
}
