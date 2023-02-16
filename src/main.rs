mod field;
mod constants;

use std::fmt::Write;
use std::{mem, slice};

use field::Field;
use constants::ARK;

pub fn as_bytes<T>(values: &[T]) -> &[u8] {
    let value_size = mem::size_of::<T>();
    let result = unsafe { slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * value_size) };
    return result;
}

// GMiMC with expanding rouding function(ERF)
struct GMiMC_erf {
  field: Field,
  capacity: u8,
  words: u8,
  round: u16,
}

impl GMiMC_erf {
  // output hash function
  fn get_hash_output(&self, value: &[u128], value_length: u8) -> [u128; 6] {
      let values = as_bytes(&value);
      let mut state = [0u128; 6];
      let state_bytes: &mut [u8; 128] =
          unsafe { &mut *(&state as *const _ as *mut [u8; 128]) };
      state_bytes[..values.len()].copy_from_slice(values);

      // TODO: improve performance
      for i in 0..self.round {
          let s0 = state[0];
          let mask = self.field.exp(self.field.add(s0, ARK[i as usize]), 3u128);
          for j in 1..512 {  // TODO: optimize iteration for performance
              state[j - 1] = self.field.add(mask, state[j]);
              if j == self.capacity as usize {
                  state[j] = s0;
                  break;
              }
          }
      }
      state
  }

  fn convert_hex_string(input: &[u8]) -> String {
      let mut byte_string = String::new();
      for b in input {
          write!(&mut byte_string, "{:x}", b).expect("Unable to write");
      }
      byte_string
  }
}


fn main() {
 let f128 = Field::new(340282366920938463463374557953744961537, 23953097886125630542083529559205016746);
 let rand = f128.rand();
 println!("random generated from f128 Field: {}", rand);
 println!("first round constant: {}", ARK[0]);
}
