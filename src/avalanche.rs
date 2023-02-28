use std::fmt::Write;
use num_bigint::{ BigUint };
use super::gmimc::{ GMiMC_erf, as_bytes };

// TODO: check performance vs using bit shift
pub const MASK: [u8; 8] = [
    0b0000_0001,
    0b0000_0010,
    0b0000_0100,
    0b0000_1000,
    0b0001_0000,
    0b0010_0000,
    0b0100_0000,
    0b1000_0000,
];

pub fn array_u128_to_u8(data: &[u128; 4]) -> [u8; 64] {
    let mut res = [0; 64];
    for i in 0..4 {
        res[16 * i..][..16].copy_from_slice(&data[i].to_be_bytes());
    }
    res
}

pub fn one_bit_changes(value: &u8) -> [u8; 8] {
    let mut shifted_values: [u8; 8] = [0u8, 0, 0, 0, 0, 0, 0, 0];
    for i in 0..8 {
        shifted_values[i] = value ^ ( 1 << i);
    }

    shifted_values
}

pub fn input_value_changes(input_value: &[u8; 64]) -> [[u8; 64]; 512] {
    let mut mutated_values = [input_value.clone(); 512];

    let mut idx = 512;
    for i in 0..=input_value.len() {
        if idx == 0 {
            break;
        } else {
            let bit_changes = one_bit_changes(&input_value[63 - i]);
            for c in bit_changes {
                idx -= 1;
                mutated_values[idx][63 - i] = c;
            }
        }
    }

    mutated_values
}

pub fn xor_u8_16(&a: &[u8; 16], &b: &[u8; 16]) -> u128 {
    let mut diff_counter = 0u128;
    for i in 0..16 {
        let xor = a[i] ^ b[i];
        diff_counter += xor.count_ones() as u128;
    }

    diff_counter
}

pub fn xor_u8_array(a: &[u8], b: &[u8]) -> BigUint {
    debug_assert!(a.len() == b.len(), "input length are different");

    let mut diff_counter = BigUint::from(0u128);
    for i in 0..a.len() {
        let xor = a[i] ^ b[i];
        diff_counter += xor.count_ones();
    }

    diff_counter
}

// TODO: all avalanche function input type as &[u8; 64]
// TODO: accept different hash_functions as input argument
pub fn calculate_avalanche_coefficient(input_value: &[u128; 4], hash_function: &GMiMC_erf) -> BigUint {
    let input_values = array_u128_to_u8(input_value);
    let input_variants = input_value_changes(&input_values);
    let mut origin_hash_output = [0u8; 64];
    hash_function.get_hash_output(as_bytes(input_value), &mut origin_hash_output);
    
    // Avalanche Co-efficient formula (guessing on my memory..)
    // M: total different cases, In here 512 variants
    // d: difference count between hashed variant input and hashed origin input_value
    // i: index of variants
    // f: hash_function
    // Avalanche-Coeffi = ( f(d_0) + f(d_1) + ... + f(d_i-1) + f(d_i) ) / M
    let mut total_different_count = BigUint::from(0u128);
    for variant in input_variants {
        let mut hash_output = [0u8; 64];
        hash_function.get_hash_output(&variant, &mut hash_output);

        // TODO: remove before commit
        let mut byte_string = String::new();
        for b in hash_output {
            write!(&mut byte_string, "{:08b}", b).expect("Unable to write");
        }
        println!("hash_output: {}", byte_string);
        
        // TODO: diif_bit_count function fixed arry to reference...
        // let diff_count = xor_u8_array(&hash_output[..32], &input_values[..32]);
        let diff_count = xor_u8_array(&hash_output, &input_values);
        println!("diff_count: {}", diff_count);
        total_different_count += BigUint::from(diff_count);

    }
    
    total_different_count / BigUint::from(512u128)
}

#[cfg(test)]
mod bit_ops {
    use super::*;
    use crate::{ field };

    #[test]
    fn diff_bit_1() {
        let a = [0b0000_0001u8, 0b0000_0011];
        let b = [0b0010_0001u8, 0b0000_0011];

        // only one difference between a and b
        let difference = xor_u8_array(&a, &b);
        assert_eq!(difference, BigUint::from(1u8));
    }

    #[test]
    fn avalanche_bit_changes() {
        // MIN & MAX
        assert_eq!([1u8, 2, 4, 8, 16, 32, 64, 128], one_bit_changes(&0u8));
        assert_eq!(
            [254u8, 253, 251, 247, 239, 223, 191, 127],
            one_bit_changes(&255u8)
        );
        assert_eq!(
            [
                0b0101_1011,
                0b0101_1000,
                0b0101_1110,
                0b0101_0010,
                0b0100_1010,
                0b0111_1010,
                0b0001_1010,
                0b1101_1010
            ],
            one_bit_changes(&90u8)
        );
        // 0b1010_0101 (165)
        assert_eq!(
            [
                0b1010_0100,
                0b1010_0111,
                0b1010_0001,
                0b1010_1101,
                0b1011_0101,
                0b1000_0101,
                0b1110_0101,
                0b0010_0101
            ],
            one_bit_changes(&165u8)
        );
    }

    #[test]
    fn avalanche_changed_input_group() {
        // the value array [0, 1, 2, 3] has four ones,
        let value = [0b000u128, 0b001, 0b0010, 0b0011];
        let input_values = array_u128_to_u8(&value);
        let changed_values = input_value_changes(&input_values);

        // Checking duplicates
        let mut has_duplicated = false;
        for p in changed_values {
            let mut du_counter = 0;
            for c in changed_values {
                if p == c { du_counter += 1 }
            }
            if du_counter > 1 {
                has_duplicated = true;
                println!("Found duplicates: {:?}", p);
            }
        }
        assert_eq!(has_duplicated, false);
    }

    #[test]
    fn avalanche_calculation() {
        let value = [0b000u128, 0b001, 0b0010, 0b0011];

        pub const M: u128 = 340282366920938463463374557953744961537;
        pub const G: u128 = 23953097886125630542083529559205016746;

        let f128 = field::Field::new(M, G);
        
        let gmimc = GMiMC_erf {
            field: f128,
            capacity: 5,
            words: 4,
            round: 166,
        };
        
        let value = [gmimc.field.rand(), gmimc.field.rand(), gmimc.field.rand(), gmimc.field.rand()];
        let result = calculate_avalanche_coefficient(&value, &gmimc);
        println!("average diffs: {:?}", result);

        assert_eq!(result >= BigUint::from(128u128), true);
    }

    // TODO: run tests at leaset 250_000 with random inputs.
    // pilot testing result in below
    // ...(1000 iteration)
    // diff_count: 240
    // total_sum_of_average: 255527 ( 49.908% =  255.527 / 512 )
    // test avalanche::bit_ops::avalanche_rand_input ... ok

    // test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 245.79s
    #[test]
    fn avalanche_rand_input() {

        pub const M: u128 = 340282366920938463463374557953744961537;
        pub const G: u128 = 23953097886125630542083529559205016746;

        let f128 = field::Field::new(M, G);
        let gmimc = GMiMC_erf {
            field: f128,
            capacity: 5,
            words: 4,
            round: 166,
        };

        let mut total_sum_of_average = BigUint::from(0u128);

        for i in 0..1000 {
            let value = [gmimc.field.rand(), gmimc.field.rand(), gmimc.field.rand(), gmimc.field.rand()];
            let result = calculate_avalanche_coefficient(&value, &gmimc);
            total_sum_of_average += result
        }
        println!("total_sum_of_average: {:?}", total_sum_of_average);
        // println!("total_sum_of_average: {:?}", total_sum_of_average / BigUint::from(10_000u128)); // That' might not good way floating
    }

}
