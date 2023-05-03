use std::fmt::Write;
use std::{mem, slice};

use crate::constants;

use ff::PrimeField;

pub fn as_bytes<T>(values: &[T]) -> &[u8] {
    let value_size = mem::size_of::<T>();
    let result =
        unsafe { slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * value_size) };
    return result;
}

// GMiMC with expanding rouding function(ERF)
#[warn(unused_allocation)]
pub struct gmimc_erf<F: PrimeField> {
    pub capacity: u8,
    pub words: u8,
    pub round: u16,
    _field: std::marker::PhantomData<F>,
}

impl<F: PrimeField> gmimc_erf<F> {
    // output hash function
    pub fn get_hash_output(&self, value: &[u128]) -> [u128; 6] {
        // TODO: remove unsafe codes
        let values = as_bytes(&value);
        let mut state = [0u128; 6];
        let state_bytes: &mut [u8; 128] = unsafe { &mut *(&state as *const _ as *mut [u8; 128]) };
        state_bytes[..values.len()].copy_from_slice(values);

        // TODO: improve performance
        for i in 0..self.round {
            let s0 = state[0];
            let a = F::from_u128(s0);
            let b = F::from_u128(constants::ARK[i as usize]);
            let mask = F::cube(&a.add(b));

            for j in 1..512 {
                // TODO: optimize iteration for performance
                let masked_state = mask + F::from_u128(state[j]);

                // Remove unsafe way to get bytes from field element
                for k in 0..16 {
                    state_bytes[k + ((j - 1) * 16)] = masked_state.to_repr().as_ref()[k]
                }
                if j == self.capacity as usize {
                    state[j] = s0;
                    break;
                }
            }
        }

        state
    }

    pub fn convert_hex_strin<T: std::fmt::LowerHex>(input: &[T]) -> String {
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
    // use crate::field;
    use ff::PrimeField;

    use super::{as_bytes, gmimc_erf};

    #[test]
    fn default_f128_hash() {
        // let f128 = field::Field::new(M, G);
        #[derive(PrimeField)]
        #[PrimeFieldModulus = "340282366920938463463374557953744961537"]
        #[PrimeFieldGenerator = "23953097886125630542083529559205016746"]
        #[PrimeFieldReprEndianness = "little"]
        struct F([u64; 3]);

        let gmimc = gmimc_erf::<F> {
            capacity: 5,
            words: 4,
            round: 166,
            _field: std::marker::PhantomData::<F>,
        };

        let value = [1u128, 2, 3, 4];
        let result = gmimc.get_hash_output(&value);

        // mimc hash result test string come from here
        // https://github.com/GuildOfWeavers/distaff/blob/fad92ce592921e671e72f93cd0078e867350860d/src/crypto/hash.rs#L293-L296
        assert_eq!(
            [
                115, 208, 64, 41, 162, 43, 134, 243, 236, 80, 161, 106, 195, 234, 30, 26, 71, 74,
                255, 77, 41, 125, 25, 152, 162, 106, 65, 108, 84, 216, 37, 37
            ],
            as_bytes(result.as_ref())[..32]
        );
    }
}
