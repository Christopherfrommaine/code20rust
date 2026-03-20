// WARNING!
// This code is pretty boilerplate. Don't bother looking at it.
// Just trust me that it implements a 256-bit integer.

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr, ShrAssign,};

#[derive(Clone, Copy, Default)]
pub struct U256 {
    v: [u64; 4]
}

#[allow(dead_code)]
pub type Int = U256;

#[allow(dead_code)]
pub const BITS: usize = 256;


#[allow(dead_code)]
pub fn to_u8(n: Int) -> u8 {
    n.v[0] as u8
}


#[allow(dead_code)]
#[inline(always)]
pub fn one() -> Int {
    U256 {v: [1, 0, 0, 0]}
}


#[allow(dead_code)]
#[inline(always)]
pub fn zero() -> Int {
    U256 {v: [0, 0, 0, 0]}
}

use std::cmp::Ordering;

impl PartialEq for Int {
    fn eq(&self, other: &Self) -> bool {
        self.v == other.v
    }
}
impl Eq for Int {}

impl PartialOrd for Int {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Int {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare limbs from most-significant to least-significant
        for i in (0..4).rev() {
            if self.v[i] < other.v[i] { return Ordering::Less; }
            if self.v[i] > other.v[i] { return Ordering::Greater; }
        }
        Ordering::Equal
    }
}

#[allow(dead_code)]
pub fn from_u128(n: u128) -> U256 {
    let lower_bit_mask = (1u128 << 64) - 1;
    let w1 = n & lower_bit_mask;
    let w2 = n & (u128::MAX ^ lower_bit_mask);

    U256 { v: [w1 as u64, w2 as u64, 0, 0] }
}

#[allow(dead_code)]
pub fn to_u128(n: Int) -> u128 {
    debug_assert!(n.v[2..].iter().copied().all(|i| i == 0), "Too large conversion encountered");
    n.v[0] as u128 + ((n.v[1] as u128) << 64)
}

#[allow(dead_code)]
pub fn mask_first_n_bits(n: usize) -> Int {
    let mut o = [0u64; 4];
    for i in 0..n {
        o[i / 64 as usize] |= 1 << (i % 64);
    }
    Int {v: o}
}

impl Int {
    pub fn count_ones(&self) -> u32 {
        let mut total = 0u32;
        for &word in &self.v {
            total += word.count_ones();
        }
        total
    }
}


impl U256 {
    pub fn from_words(words: [u64; 4]) -> Self {
        Self { v: words }
    }
}

use std::fmt;

impl fmt::Display for U256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.v.iter().all(|&x| x == 0) {
            return write!(f, "0");
        }

        let mut n = *self;
        let mut digits = Vec::new();

        while n.v.iter().any(|&x| x != 0) {
            let mut remainder = 0u64;
            for i in (0..4).rev() {
                let value = ((remainder as u128) << 64) | (n.v[i] as u128);
                n.v[i] = (value / 10) as u64;
                remainder = (value % 10) as u64;
            }
            digits.push(remainder as u8);
        }

        for d in digits.iter().rev() {
            write!(f, "{}", d)?;
        }

        Ok(())
    }
}
impl fmt::Debug for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}


impl BitAnd for U256 {
    type Output = U256;
    fn bitand(self, rhs: U256) -> U256 {
        U256::from_words([
            self.v[0] & rhs.v[0],
            self.v[1] & rhs.v[1],
            self.v[2] & rhs.v[2],
            self.v[3] & rhs.v[3],
        ])
    }
}
impl BitOr for U256 {
    type Output = U256;
    fn bitor(self, rhs: U256) -> U256 {
        U256::from_words([
            self.v[0] | rhs.v[0],
            self.v[1] | rhs.v[1],
            self.v[2] | rhs.v[2],
            self.v[3] | rhs.v[3],
        ])
    }
}
impl BitXor for U256 {
    type Output = U256;
    fn bitxor(self, rhs: U256) -> U256 {
        U256::from_words([
            self.v[0] ^ rhs.v[0],
            self.v[1] ^ rhs.v[1],
            self.v[2] ^ rhs.v[2],
            self.v[3] ^ rhs.v[3],
        ])
    }
}
impl Not for U256 {
    type Output = U256;
    fn not(self) -> U256 {
        U256::from_words([!self.v[0], !self.v[1], !self.v[2], !self.v[3]])
    }
}

/* Assign variants */
impl BitAndAssign for U256 {
    fn bitand_assign(&mut self, rhs: U256) {
        for i in 0..4 {
            self.v[i] &= rhs.v[i];
        }
    }
}
impl BitOrAssign for U256 {
    fn bitor_assign(&mut self, rhs: U256) {
        for i in 0..4 {
            self.v[i] |= rhs.v[i];
        }
    }
}
impl BitXorAssign for U256 {
    fn bitxor_assign(&mut self, rhs: U256) {
        for i in 0..4 {
            self.v[i] ^= rhs.v[i];
        }
    }
}

/* Shifts */
fn shl_words(src: &[u64; 4], n: usize) -> [u64; 4] {
    if n >= 256 {
        return [0; 4];
    }
    let word_shift = n / 64;
    let bit_shift = n % 64;
    let mut out = [0u64; 4];

    if bit_shift == 0 {
        for i in (word_shift..4).rev() {
            out[i] = src[i - word_shift];
        }
        return out;
    }

    for i in (0..4).rev() {
        let src_idx = i as isize - word_shift as isize;
        if src_idx < 0 {
            out[i] = 0;
        } else {
            let lo = src[src_idx as usize] << bit_shift;
            let hi = if src_idx as usize >= 1 {
                src[src_idx as usize - 1] >> (64 - bit_shift)
            } else {
                0
            };
            out[i] = lo | hi;
        }
    }
    out
}

fn shr_words(src: &[u64; 4], n: usize) -> [u64; 4] {
    if n >= 256 {
        return [0; 4];
    }
    let word_shift = n / 64;
    let bit_shift = n % 64;
    let mut out = [0u64; 4];

    if bit_shift == 0 {
        for i in 0..(4 - word_shift) {
            out[i] = src[i + word_shift];
        }
        return out;
    }

    for i in 0..4 {
        let src_idx = i + word_shift;
        if src_idx >= 4 {
            out[i] = 0;
        } else {
            let lo = src[src_idx] >> bit_shift;
            let hi = if src_idx + 1 < 4 {
                src[src_idx + 1] << (64 - bit_shift)
            } else {
                0
            };
            out[i] = lo | hi;
        }
    }
    out
}

impl Shl<usize> for U256 {
    type Output = U256;
    fn shl(self, rhs: usize) -> U256 {
        U256::from_words(shl_words(&self.v, rhs))
    }
}
impl Shr<usize> for U256 {
    type Output = U256;
    fn shr(self, rhs: usize) -> U256 {
        U256::from_words(shr_words(&self.v, rhs))
    }
}
impl ShlAssign<usize> for U256 {
    fn shl_assign(&mut self, rhs: usize) {
        self.v = shl_words(&self.v, rhs);
    }
}
impl ShrAssign<usize> for U256 {
    fn shr_assign(&mut self, rhs: usize) {
        self.v = shr_words(&self.v, rhs);
    }
}


impl Shl<u32> for U256 {
    type Output = U256;
    fn shl(self, rhs: u32) -> U256 {
        U256::from_words(shl_words(&self.v, rhs as usize))
    }
}
impl Shr<u32> for U256 {
    type Output = U256;
    fn shr(self, rhs: u32) -> U256 {
        U256::from_words(shr_words(&self.v, rhs as usize))
    }
}
impl ShlAssign<u32> for U256 {
    fn shl_assign(&mut self, rhs: u32) {
        self.v = shl_words(&self.v, rhs as usize);
    }
}
impl ShrAssign<u32> for U256 {
    fn shr_assign(&mut self, rhs: u32) {
        self.v = shr_words(&self.v, rhs as usize);
    }
}

/* Some convenience impls for references to avoid moves */
impl<'a> BitAnd for &'a U256 {
    type Output = U256;
    fn bitand(self, rhs: &'a U256) -> U256 {
        *self & *rhs
    }
}
impl<'a> BitOr for &'a U256 {
    type Output = U256;
    fn bitor(self, rhs: &'a U256) -> U256 {
        *self | *rhs
    }
}
impl<'a> BitXor for &'a U256 {
    type Output = U256;
    fn bitxor(self, rhs: &'a U256) -> U256 {
        *self ^ *rhs
    }
}


impl U256 {
    pub fn trailing_zeros(&self) -> u32 {
        for i in 0..4 {
            if self.v[i] != 0 {
                return (i as u32) * 64 + self.v[i].trailing_zeros();
            }
        }
        256
    }

    pub fn reverse_bits(&self) -> U256 {
        U256 {
            v: [
                self.v[3].reverse_bits(),
                self.v[2].reverse_bits(),
                self.v[1].reverse_bits(),
                self.v[0].reverse_bits(),
            ],
        }
    }

    pub fn min(self, other: U256) -> U256 {
        for i in (0..4).rev() {
            if self.v[i] < other.v[i] {
                return self;
            } else if self.v[i] > other.v[i] {
                return other;
            }
        }
        self
    }

    pub fn ilog2(&self) -> u32 {
        for i in (0..4).rev() {
            if self.v[i] != 0 {
                return (i as u32) * 64 + 63 - self.v[i].leading_zeros();
            }
        }
        panic!("ilog2 called on zero");
    }
}
