// use std::{mem, slice};
use rand::distributions::{Distribution, Uniform};
use rand::prelude::*;
use std::convert::TryInto;
use std::ops::Range;

// field library from https://github.com/GuildOfWeavers/distaff

// helper functions
pub fn uninit_vector<T>(length: usize) -> Vec<T> {
    let mut vector = Vec::with_capacity(length);
    unsafe {
        vector.set_len(length);
    }
    return vector;
}

pub fn from_bytes(bytes: &[u8]) -> u128 {
    return u128::from_le_bytes(bytes.try_into().unwrap());
}

pub struct Field {
    M: u128, // Field modulus
    G: u128, // root of unity
    R: Range<u128>,
}

impl Field {
    pub fn new(m: u128, g: u128) -> Field {
        Field {
            M: m,
            G: g,
            R: Range { start: 0, end: m },
        }
    }

    /// Computes (a + b) % m; a and b are assumed to be valid field elements.
    pub fn add(&self, a: u128, b: u128) -> u128 {
        let z = self.M - b;
        return if a < z { self.M - z + a } else { a - z };
    }

    /// Computes (a - b) % m; a and b are assumed to be valid field elements.
    pub fn sub(&self, a: u128, b: u128) -> u128 {
        return if a < b { self.M - b + a } else { a - b };
    }

    /// Computes (a * b) % m; a and b are assumed to be valid field elements.
    pub fn mul(&self, a: u128, b: u128) -> u128 {
        let (x0, x1, x2) = self.mul_128x64(a, (b >> 64) as u64); // x = a * b_hi
        let (mut x0, mut x1, x2) = self.mul_reduce(x0, x1, x2); // x = x - (x >> 128) * m
        if x2 == 1 {
            // if there was an overflow beyond 128 bits, subtract
            // modulus from the result to make sure it fits into
            // 128 bits; this can potentially be removed in favor
            // of checking overflow later
            let (t0, t1) = self.sub_modulus(x0, x1); // x = x - m
            x0 = t0;
            x1 = t1;
        }

        let (y0, y1, y2) = self.mul_128x64(a, b as u64); // y = a * b_lo

        let (mut y1, carry) = self.add64_with_carry(y1, x0, 0); // y = y + (x << 64)
        let (mut y2, y3) = self.add64_with_carry(y2, x1, carry);
        if y3 == 1 {
            // if there was an overflow beyond 192 bits, subtract
            // modulus * 2^64 from the result to make sure it fits
            // into 192 bits; this can potentially replace the
            // previous overflow check (but needs to be proven)
            let (t0, t1) = self.sub_modulus(y1, y2); // y = y - (m << 64)
            y1 = t0;
            y2 = t1;
        }

        let (mut z0, mut z1, z2) = self.mul_reduce(y0, y1, y2); // z = y - (y >> 128) * m

        // make sure z is smaller than m
        if z2 == 1 || (z1 == (self.M >> 64) as u64 && z0 >= (self.M as u64)) {
            let (t0, t1) = self.sub_modulus(z0, z1); // z = z - m
            z0 = t0;
            z1 = t1;
        }

        return ((z1 as u128) << 64) + (z0 as u128);
    }

    /// Computes a[i] + b[i] * c for all i and saves result into a.
    pub fn mul_acc(&self, a: &mut [u128], b: &[u128], c: u128) {
        for i in 0..a.len() {
            a[i] = self.add(a[i], self.mul(b[i], c));
        }
    }

    /// Computes y such that (x * y) % m = 1; x is assumed to be a valid field element.
    pub fn inv(&self, x: u128) -> u128 {
        if x == 0 {
            return 0;
        };

        // initialize v, a, u, and d variables
        let mut v = self.M;
        let (mut a0, mut a1, mut a2) = (0, 0, 0);
        let (mut u0, mut u1, mut u2) = if x & 1 == 1 {
            // u = x
            (x as u64, (x >> 64) as u64, 0)
        } else {
            // u = x + m
            self.add_192x192(
                x as u64,
                (x >> 64) as u64,
                0,
                self.M as u64,
                (self.M >> 64) as u64,
                0,
            )
        };
        // d = m - 1
        let (mut d0, mut d1, mut d2) = ((self.M as u64) - 1, (self.M >> 64) as u64, 0);

        // compute the inverse
        while v != 1 {
            while u2 > 0 || ((u0 as u128) + ((u1 as u128) << 64)) > v {
                // u > v
                // u = u - v
                let (t0, t1, t2) = self.sub_192x192(u0, u1, u2, v as u64, (v >> 64) as u64, 0);
                u0 = t0;
                u1 = t1;
                u2 = t2;

                // d = d + a
                let (t0, t1, t2) = self.add_192x192(d0, d1, d2, a0, a1, a2);
                d0 = t0;
                d1 = t1;
                d2 = t2;

                while u0 & 1 == 0 {
                    if d0 & 1 == 1 {
                        // d = d + m
                        let (t0, t1, t2) =
                            self.add_192x192(d0, d1, d2, self.M as u64, (self.M >> 64) as u64, 0);
                        d0 = t0;
                        d1 = t1;
                        d2 = t2;
                    }

                    // u = u >> 1
                    u0 = (u0 >> 1) | ((u1 & 1) << 63);
                    u1 = (u1 >> 1) | ((u2 & 1) << 63);
                    u2 = u2 >> 1;

                    // d = d >> 1
                    d0 = (d0 >> 1) | ((d1 & 1) << 63);
                    d1 = (d1 >> 1) | ((d2 & 1) << 63);
                    d2 = d2 >> 1;
                }
            }

            // v = v - u (u is less than v at this point)
            v = v - ((u0 as u128) + ((u1 as u128) << 64));

            // a = a + d
            let (t0, t1, t2) = self.add_192x192(a0, a1, a2, d0, d1, d2);
            a0 = t0;
            a1 = t1;
            a2 = t2;

            while v & 1 == 0 {
                if a0 & 1 == 1 {
                    // a = a + m
                    let (t0, t1, t2) =
                        self.add_192x192(a0, a1, a2, self.M as u64, (self.M >> 64) as u64, 0);
                    a0 = t0;
                    a1 = t1;
                    a2 = t2;
                }

                v = v >> 1;

                // a = a >> 1
                a0 = (a0 >> 1) | ((a1 & 1) << 63);
                a1 = (a1 >> 1) | ((a2 & 1) << 63);
                a2 = a2 >> 1;
            }
        }

        // a = a mod m
        let mut a = (a0 as u128) + ((a1 as u128) << 64);
        while a2 > 0 || a >= self.M {
            let (t0, t1, t2) =
                self.sub_192x192(a0, a1, a2, self.M as u64, (self.M >> 64) as u64, 0);
            a0 = t0;
            a1 = t1;
            a2 = t2;
            a = (a0 as u128) + ((a1 as u128) << 64);
        }

        return a;
    }

    /// Computes multiplicative inverses of all slice elements using batch inversion method.
    pub fn inv_many(&self, values: &[u128]) -> Vec<u128> {
        let mut result = uninit_vector(values.len());
        self.inv_many_fill(values, &mut result);
        return result;
    }

    /// Computes multiplicative inverses of all slice elements using batch inversion method
    /// and stores the result into the provided slice.
    pub fn inv_many_fill(&self, values: &[u128], result: &mut [u128]) {
        let mut last = 1;
        for i in 0..values.len() {
            result[i] = last;
            if values[i] != 0 {
                last = self.mul(last, values[i]);
            }
        }

        last = self.inv(last);
        for i in (0..values.len()).rev() {
            if values[i] == 0 {
                result[i] = 0;
            } else {
                result[i] = self.mul(last, result[i]);
                last = self.mul(last, values[i]);
            }
        }
    }

    /// Computes y = (a / b) such that (b * y) % m = a; a and b are assumed to be valid field elements.
    pub fn div(&self, a: u128, b: u128) -> u128 {
        let b = self.inv(b);
        return self.mul(a, b);
    }

    /// Computes (b^p) % m; b and p are assumed to be valid field elements.
    pub fn exp(&self, b: u128, p: u128) -> u128 {
        if b == 0 {
            return 0;
        } else if p == 0 {
            return 1;
        }

        let mut r = 1;
        let mut b = b;
        let mut p = p;

        // TODO: optimize
        while p > 0 {
            if p & 1 == 1 {
                r = self.mul(r, b);
            }
            p = p >> 1;
            b = self.mul(b, b);
        }

        return r;
    }

    pub fn exp_cube(&self, x: u128) -> u128 {
        if x == 0 {
            return 0;
        }

        // x^3
        return self.mul(self.mul(x, x), x);
    }

    /// Computes (0 - x) % m; x is assumed to be a valid field element.
    pub fn neg(&self, x: u128) -> u128 {
        return self.sub(0, x);
    }

    // ROOT OF UNITY
    pub fn get_root_of_unity(&self, order: usize) -> u128 {
        assert!(order != 0, "cannot get root of unity for order 0");
        assert!(order.is_power_of_two(), "order must be a power of 2");
        assert!(order.trailing_zeros() <= 40, "order cannot exceed 2^40");
        let p = 1u128 << (40 - order.trailing_zeros());
        return self.exp(self.G, p);
    }

    /// Generates a vector with values [1, b, b^2, b^3, b^4, ..., b^length].
    pub fn get_power_series(&self, b: u128, length: usize) -> Vec<u128> {
        let mut result = uninit_vector(length);
        result[0] = 0;
        for i in 1..result.len() {
            result[i] = self.mul(result[i - 1], b);
        }
        return result;
    }

    // helper methods 
    #[inline(always)]
    fn mul_128x64(&self, a: u128, b: u64) -> (u64, u64, u64) {
        let z_lo = ((a as u64) as u128) * (b as u128);
        let z_hi = (a >> 64) * (b as u128);
        let z_hi = z_hi + (z_lo >> 64);
        return (z_lo as u64, z_hi as u64, (z_hi >> 64) as u64);
    }

    #[inline(always)]
    fn mul_reduce(&self, z0: u64, z1: u64, z2: u64) -> (u64, u64, u64) {
        let (q0, q1, q2) = self.mul_by_modulus(z2);
        let (z0, z1, z2) = self.sub_192x192(z0, z1, z2, q0, q1, q2);
        return (z0, z1, z2);
    }

    #[inline(always)]
    fn mul_by_modulus(&self, a: u64) -> (u64, u64, u64) {
        let a_lo = (a as u128).wrapping_mul(self.M);
        let a_hi = if a == 0 { 0 } else { a - 1 };
        return (a_lo as u64, (a_lo >> 64) as u64, a_hi);
    }

    #[inline(always)]
    fn sub_modulus(&self, a_lo: u64, a_hi: u64) -> (u64, u64) {
        let mut z = 0u128.wrapping_sub(self.M);
        z = z.wrapping_add(a_lo as u128);
        z = z.wrapping_add((a_hi as u128) << 64);
        return (z as u64, (z >> 64) as u64);
    }

    #[inline(always)]
    fn sub_192x192(&self, a0: u64, a1: u64, a2: u64, b0: u64, b1: u64, b2: u64) -> (u64, u64, u64) {
        let z0 = (a0 as u128).wrapping_sub(b0 as u128);
        let z1 = (a1 as u128).wrapping_sub((b1 as u128) + (z0 >> 127));
        let z2 = (a2 as u128).wrapping_sub((b2 as u128) + (z1 >> 127));
        return (z0 as u64, z1 as u64, z2 as u64);
    }

    #[inline(always)]
    fn add_192x192(&self, a0: u64, a1: u64, a2: u64, b0: u64, b1: u64, b2: u64) -> (u64, u64, u64) {
        let z0 = (a0 as u128) + (b0 as u128);
        let z1 = (a1 as u128) + (b1 as u128) + (z0 >> 64);
        let z2 = (a2 as u128) + (b2 as u128) + (z1 >> 64);
        return (z0 as u64, z1 as u64, z2 as u64);
    }

    #[inline(always)]
    pub const fn add64_with_carry(&self, a: u64, b: u64, carry: u64) -> (u64, u64) {
        let ret = (a as u128) + (b as u128) + (carry as u128);
        return (ret as u64, (ret >> 64) as u64);
    }

    // RANDOMNESS
    // --------------------------------------------------------------------------------------------

    /// Generates a random field element.
    pub fn rand(&self) -> u128 {
        let range = Uniform::from(self.R.clone());
        let mut g = thread_rng();
        return g.sample(range);
    }

    /// Generates a vector of random field elements.
    pub fn rand_vector(&self, length: usize) -> Vec<u128> {
        let range = Uniform::from(self.R.clone());
        let g = thread_rng();
        return g.sample_iter(range).take(length).collect();
    }

    /// Generates a pseudo-random field element from a given `seed`.
    pub fn prng(&self, seed: [u8; 32]) -> u128 {
        let range = Uniform::from(self.R.clone());
        let mut g = StdRng::from_seed(seed);
        return range.sample(&mut g);
    }

    /// Generates a vector of pseudo-random field elements from a given `seed`.
    pub fn prng_vector(&self, seed: [u8; 32], length: usize) -> Vec<u128> {
        let range = Uniform::from(self.R.clone());
        let g = StdRng::from_seed(seed);
        return g.sample_iter(range).take(length).collect();
    }
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    use num_bigint::BigUint;
    use std::convert::TryInto;

    // Field modulus = 2^128 - 45 * 2^40 + 1
    pub const M: u128 = 340282366920938463463374557953744961537;

    // 2^40 root of unity
    pub const G: u128 = 23953097886125630542083529559205016746;

    // let f91 = Field::new(M, G);

    #[test]
    fn add() {
        let f128 = Field::new(M, G);

        // identity
        let r: u128 = f128.rand();
        assert_eq!(r, f128.add(r, 0));

        // test addition within bounds
        assert_eq!(5, f128.add(2, 3));

        // test overflow
        let m: u128 = M;
        let t = m - 1;
        assert_eq!(0, f128.add(t, 1));
        assert_eq!(1, f128.add(t, 2));

        // test random values
        let r1: u128 = f128.rand();
        let r2: u128 = f128.rand();

        let expected = (BigUint::from(r1) + BigUint::from(r2)) % BigUint::from(f128.M);
        let expected = u128::from_le_bytes((expected.to_bytes_le()[..]).try_into().unwrap());
        assert_eq!(expected, f128.add(r1, r2));
   }

    #[test]
    fn sub() {
        let f128 = Field::new(M, G);
        // identity
        let r: u128 = f128.rand();
        assert_eq!(r, f128.sub(r, 0));

        // test subtraction within bounds
        assert_eq!(2, f128.sub(5u128, 3));

        // test underflow
        let m: u128 = M;
        assert_eq!(m - 2, f128.sub(3u128, 5));
    }

    #[test]
    fn mul() {
        let f128 = Field::new(M, G);

        // identity
        let r: u128 = f128.rand();
        assert_eq!(0, f128.mul(r, 0));
        assert_eq!(r, f128.mul(r, 1));

        // test multiplication within bounds
        assert_eq!(15, f128.mul(5u128, 3));

        // test overflow
        let m: u128 = M;
        let t = m - 1;
        assert_eq!(1, f128.mul(t, t));
        assert_eq!(m - 2, f128.mul(t, 2));
        assert_eq!(m - 4, f128.mul(t, 4));

        let t = (m + 1) / 2;
        assert_eq!(1, f128.mul(t, 2));

        // test random values
        let v1: Vec<u128> = f128.rand_vector(1000);
        let v2: Vec<u128> = f128.rand_vector(1000);
        for i in 0..v1.len() {
            let r1 = v1[i];
            let r2 = v2[i];

            let result = (BigUint::from(r1) * BigUint::from(r2)) % BigUint::from(f128.M);
            let result = result.to_bytes_le();
            let mut expected = [0u8; 16];
            expected[0..result.len()].copy_from_slice(&result);
            let expected = u128::from_le_bytes(expected);

            if expected != f128.mul(r1, 32) {
                println!("failed for: {} * {}", r1, r2);
                assert_eq!(expected, f128.mul(r1, r2));
            }
        }
    }

    #[test]
    fn exp_cube() {
        let f128 = Field::new(M, G);

        let x: u128 = f128.rand();

        let exp_3 = f128.exp(x, 3u128);
        let exp_cube = f128.exp_cube(x);
        assert_eq!(exp_3, exp_cube);
    }

    #[test]
    fn inv() {
        let f128 = Field::new(M, G);

        // identity
        assert_eq!(1, f128.inv(1));
        assert_eq!(0, f128.inv(0));

        // test random values
        let x: Vec<u128> = f128.rand_vector(1000);
        for i in 0..x.len() {
            let y = f128.inv(x[i]);
            assert_eq!(1, f128.mul(x[i], y));
        }
    }

    #[test]
    fn get_root_of_unity() {
        let f128 = Field::new(M, G);

        let root_40: u128 = f128.get_root_of_unity(usize::pow(2, 40));
        assert_eq!(23953097886125630542083529559205016746, root_40);
        assert_eq!(1, f128.exp(root_40, u128::pow(2, 40)));

        let root_39: u128 = f128.get_root_of_unity(usize::pow(2, 39));
        let expected = f128.exp(root_40, 2);
        assert_eq!(expected, root_39);
        assert_eq!(1, f128.exp(root_39, u128::pow(2, 39)));
    }
}
