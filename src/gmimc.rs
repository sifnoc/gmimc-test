use std::fmt::Write;
use std::{mem, slice};

use crate::{field, constants};

pub fn as_bytes<T>(values: &[T]) -> &[u8] {
    let value_size = mem::size_of::<T>();
    let result =
        unsafe { slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * value_size) };
    return result;
}

// GMiMC with expanding rouding function(ERF)
pub struct GMiMC_erf {
    pub field: field::Field,
    pub capacity: u8,
    pub words: u8,
    pub round: u16,
}

impl GMiMC_erf {
    // output hash function
    pub fn get_hash_output(&self, value: &[u128]) -> [u128; 6] {
        let values = as_bytes(&value);
        let mut state = [0u128; 6];
        let state_bytes: &mut [u8; 128] = unsafe { &mut *(&state as *const _ as *mut [u8; 128]) };
        state_bytes[..values.len()].copy_from_slice(values);

        // TODO: improve performance
        for i in 0..self.round {
            let s0 = state[0];
            let mask = self.field.exp_cube(self.field.add(s0, constants::ARK[i as usize]));
            for j in 1..512 {
                // TODO: optimize iteration for performance
                state[j - 1] = self.field.add(mask, state[j]);
                if j == self.capacity as usize {
                    state[j] = s0;
                    break;
                }
            }
        }

        state
    }

    pub fn convert_hex_string(input: &[u8]) -> String {
        let mut byte_string = String::new();
        for b in input {
            write!(&mut byte_string, "{:x}", b).expect("Unable to write");
        }
        byte_string
    }
}

// Testing
#[cfg(test)]
mod unit {
    use crate::field;

    use super::as_bytes;
    use super::GMiMC_erf;

    pub const M: u128 = 340282366920938463463374557953744961537;
    pub const G: u128 = 23953097886125630542083529559205016746;

    #[test]
    fn default_f128_hash() {
        let f128 = field::Field::new(M, G);
        let gmimc = GMiMC_erf {
            field: f128,
            capacity: 5,
            words: 4,
            round: 166,
        };

        let value = [1u128, 2, 3, 4];
        let result = gmimc.get_hash_output(&value);
        assert_eq!(
            [
                115, 208, 64, 41, 162, 43, 134, 243, 236, 80, 161, 106, 195, 234, 30, 26, 71, 74,
                255, 77, 41, 125, 25, 152, 162, 106, 65, 108, 84, 216, 37, 37
            ],
            as_bytes(&result[..2])
        )
    }
}
