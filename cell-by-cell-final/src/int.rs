/// This file just reditects all the functions and constants
/// to one of the other uint files. To change the size of the uint,
/// just change all occourances of 256 to 1024, or 128 to 256, or whatever
/// it is currently to whatever you want it to be.

use bnum::types::*;

pub type Int = U1024;

pub const BITS: usize = Int::BITS as usize;

pub fn to_u8(n: Int) -> u8 {
    to_u128(n) as u8
}

#[inline(always)]
pub fn one() -> Int {
    Int::ONE
}

#[inline(always)]
pub fn zero() -> Int {
    Int::ZERO
}

#[allow(dead_code)]
#[inline(always)]
pub fn from_u128(n: u128) -> Int {
    Int::from_digit(n as u64)
}

#[allow(dead_code)]
#[inline(always)]
pub fn to_u128(n: Int) -> u128 {
    *(n.digits().first().unwrap()) as u128
}

static MASK_FIRST_BITS_CACHE: std::sync::LazyLock<Vec<Int>> = std::sync::LazyLock::new(|| 
    (0..BITS).map(|n| {
        let mut o = [0u64; BITS / 64];
        let max = n / 64;
        for i in 0..max {
            o[i] = u64::MAX
        }
        for i in (64 * max)..n {
            o[max] |= 1 << (i % 64)
        }
        
        Int::from_digits(o)
    }).collect::<Vec<Int>>()
);

// #[inline(always)]
// pub fn mask_first_bits(n: usize) -> Int {
//     let mut o = [0u64; BITS / 64];
//     let max = n / 64;
//     for i in 0..max {
//         o[i] = u64::MAX
//     }
//     for i in (64 * max)..n {
//         o[max] |= 1 << (i % 64)
//     }
//     
//     Int::from_digits(o)
// }

#[inline(always)]
pub fn mask_first_bits(n: usize) -> Int {
    MASK_FIRST_BITS_CACHE[n]
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    fn check_eq_builtin(x: u128, y: u128) {
        let a = from_u128(x);
        let b = from_u128(y);

        // Bitwise ops
        assert_eq!(from_u128(x & y), a & b, "AND failed for {x} & {y}");
        assert_eq!(from_u128(x | y), a | b, "OR failed for {x} | {y}");
        assert_eq!(from_u128(x ^ y), a ^ b, "XOR failed for {x} ^ {y}");

        // Shifts — especially near 0, 63, 64, 65, 127
        for k in [0u32, 1, 63, 64, 65, 127] {
            let max_shift = 128 - x.ilog2();
            if k >= max_shift {continue;}

            assert_eq!(from_u128(x << k), a << k, "SHL failed for {x} << {k}");
            assert_eq!(from_u128(x >> k), a >> k, "SHR failed for {x} >> {k}");
        }

        // Comparisons
        assert_eq!(x == y, a == b, "EQ failed for {x} vs {y}");
        assert_eq!(x < y,  a < b,  "LT failed for {x} vs {y}");
        assert_eq!(x > y,  a > b,  "GT failed for {x} vs {y}");

        // Trailing zeros + ilog2 if implemented
        assert_eq!(x.trailing_zeros(), a.trailing_zeros(), "trailing_zeros failed for {x}");
        if x > 0 {
            assert_eq!(x.ilog2(), a.ilog2(), "ilog2 failed for {x}");
        }

        // Reverse bits (mask down to actual bit size)
        // assert_eq!(from_u128(x.reverse_bits()), a.reverse_bits(), "reverse_bits failed for {x}");
    }

    #[test]
    fn fuzz_all_operations_against_u128() {
        let mut rng: rand::prelude::ThreadRng = rand::rng();
        for _ in 0..100_000 {
            let x: u128 = rng.random();
            let y: u128 = rng.random();
            check_eq_builtin(x, y);
        }
    }

    #[test]
    fn check_shr_small() {
        let x: u128 = 1 << 100;
        let big = from_u128(x);
        assert_eq!(from_u128(x >> 1), big >> 1u32);
    }

    #[test]
    fn shifts_across_limb_boundary() {
        // Choose values that straddle 64-bit boundaries.
        let cases: [u128; 6] = [
            (1u128 << 63) | 1,              // exactly at 63
            (1u128 << 64) | 3,              // crosses 64
            (1u128 << 65) | (1u128 << 1),
            (1u128 << 100) | (1u128 << 63), // crosses 63/64 and 100
            u128::MAX - 1,
            0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210u128,
        ];

        for &x in &cases {
            let a = from_u128(x);

            for k in [1usize, 2, 3, 7, 31, 32, 33, 63, 64, 65, 100, 127] {
                let max_shift = 128 - x.ilog2();
                if k >= max_shift as usize {continue;}

                // SHR
                let expected = from_u128(x >> k);
                let got = a >> k;
                assert_eq!(expected, got, "SHR failed for {x} >> {k}");

                // SHL (mask to u128 width)
                let expected = from_u128(x.wrapping_shl(k as u32));
                let got = a << k;
                assert_eq!(expected, got, "SHL failed for {x} << {k}");
            }
        }
    }

    #[test]
    fn fuzz_shifts_vs_u128() {
        use rand::Rng;
        let mut rng = rand::rng();
        for _ in 0..20_000 {
            let x: u128 = rng.random();
            let a = from_u128(x);

            let max_shift = 128 - x.ilog2();
    
            // Right-shifts 1..127
            let k = rng.random_range(1..128usize);
            if k >= max_shift as usize {continue;}
            assert_eq!(from_u128(x >> k), a >> k, "SHR fuzz {x} >> {k}");
    
            // Left-shifts 0..127 (wrapping within 128-bit window)
            let k = rng.random_range(0..128usize);
            if k >= max_shift as usize {continue;}
            assert_eq!(from_u128(x.wrapping_shl(k as u32)), a << k, "SHL fuzz {x} << {k}");
        }
    }

    #[test]
    fn test_incrementing_shifts() {
        (1u128..1_000).for_each(|x| {
            let xi = from_u128(x);

            for s in 0..(127 - xi.ilog2()) {
                assert_eq!(to_u128(xi << s), x << s, "SHL");
                assert_eq!(to_u128(xi >> s), x >> s, "SHR");
            }
        });
    }

    #[test]
    fn test_power_of_two_shifts() {

        (0u128..127).for_each(|i| {
            let x = 1u128 << i;
            let xi = from_u128(x);

            println!("{x}");

            for s in 0..(127 - xi.ilog2()) {
                assert_eq!(to_u128(xi << s), x << s, "SHL");
                assert_eq!(to_u128(xi >> s), x >> s, "SHR");
            }
        });
    }

    #[test]
    fn test_first_n_bits_mask() {
        assert_eq!(to_u128(mask_first_bits(5)), (1 << 5) - 1);
    }

    #[test]
    fn test_u128_conversion_equality() {

        (0u128..127).for_each(|i| {
            let x = 1u128 << i;
            let xi = to_u128(from_u128(x));

            assert_eq!(x, xi);
        });

        (1u128..1_000).for_each(|x| {
            let xi = to_u128(from_u128(x));

            assert_eq!(x, xi);
        });
    }
}
