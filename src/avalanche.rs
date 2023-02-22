use std::fmt::Write;
// use crate::gmimc::as_bytes;

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
        if value & MASK[i] == 0u8 {
            shifted_values[i] = value + (1 << i);
        } else {
            shifted_values[i] = value - (value & MASK[i]);
        }
    }

    shifted_values
}

pub fn input_value_changes(input_value: &[u128; 4]) -> [[u8; 64]; 512] {
    let values = array_u128_to_u8(input_value);
    let mut mutated_values = [values.clone(); 512];

    let mut idx = 512;
    for i in 0..=values.len() {
        if idx == 0 {
            break;
        } else {
            let bit_changes = one_bit_changes(&values[63 - i]);
            for c in bit_changes {
                idx -= 1;
                mutated_values[idx][63 - i] = c;
            }
        }
    }

    mutated_values
}

pub fn diff_bit_count(input_value: &[u8; 64], hash_output: [u8; 64]) -> u128 {
    let mut diff_counter = 0u128;
    for i in 0..64 {
        let xor = input_value[i] ^ hash_output[i];
        diff_counter += xor.count_ones() as u128;
    }

    diff_counter
}

#[cfg(test)]
mod bit_ops {
    use super::*;

    #[test]
    fn bit_changes() {
        // MIN & MAX
        assert_eq!([1u8, 2, 4, 8, 16, 32, 64, 128], one_bit_changes(&0u8));
        assert_eq!(
            [254u8, 253, 251, 247, 239, 223, 191, 127],
            one_bit_changes(&255u8)
        );

        // 0b0101_1010 (90)
        //  >> 0
        //   val: 01011010
        //  mask: 00000001
        //      > 00000000
        //  mutd: 01011011
        //  =============
        //  >> 1
        //   val: 01011010
        //  mask: 00000010
        //      > 00000010
        //  mutd: 01011000
        //  =============
        //  >> 2
        //   val: 01011010
        //  mask: 00000100
        //      > 00000000
        //  mutd: 01011110
        //  =============
        //  >> 3
        //   val: 01011010
        //  mask: 00001000
        //      > 00001000
        //  mutd: 01010010
        //  =============
        //  >> 4
        //   val: 01011010
        //  mask: 00010000
        //      > 00010000
        //  mutd: 01001010
        //  =============
        //  >> 5
        //   val: 01011010
        //  mask: 00100000
        //      > 00000000
        //  mutd: 01111010
        //  =============
        //  >> 6
        //   val: 01011010
        //  mask: 01000000
        //      > 01000000
        //  mutd: 00011010
        //  =============
        //  >> 7
        //   val: 01011010
        //  mask: 10000000
        //      > 00000000
        //  mutd: 11011010
        //  =============
        assert_eq!(
            [
                0b0101_1011,
                0b0101_1000,
                0b0101_1110,
                0b010_10010,
                0b0100_1010,
                0b0111_1010,
                0b000_11010,
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
    fn changed_input_group() {
        // the value array [0, 1, 2, 3] has four ones,
        let value = [0b000u128, 0b001, 0b0010, 0b0011];
        let changed_values = input_value_changes(&value);
        
        // // For checking binary digit movement
        // for elem in changed_values.iter().enumerate() {
        //     let mut byte_string = String::new();
        //     let (i, e) = elem;

        //     let mut one_counter = 0u128;
        //     for d in e {
        //         one_counter += d.count_ones() as u128;
        //     }

            
        //     for b in e {
        //         write!(&mut byte_string, "{:08b}_", b).expect("Unable to write");
        //     }

        //     println!("{} count 1: {}\n{:?}", i, one_counter, byte_string);
        // }

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
}
