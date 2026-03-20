use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr, ShrAssign};

#[derive(Clone, Copy, Default, Hash)]
pub struct U1024 {
    pub v: [u64; 16],
}


#[allow(dead_code)]
pub type Int = U1024;


#[allow(dead_code)]
pub const BITS: usize = 1024;


#[allow(dead_code)]
pub fn to_u8(n: Int) -> u8 {
    n.v[0] as u8
}


#[allow(dead_code)]
#[inline(always)]
pub fn one() -> Int {
    U1024 { v: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }
}


#[allow(dead_code)]
pub fn from_u128(n: u128) -> U1024 {
    let w1 = n as u64;
    let w2 = (n >> 64) as u64;

    U1024 { v: [w1, w2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }
}

#[allow(dead_code)]
pub fn to_u128(n: Int) -> u128 {
    // debug_assert!(n.v[2..].iter().copied().all(|i| i == 0));
    n.v[0] as u128 + ((n.v[1] as u128) << 64)
}

#[allow(dead_code)]
pub fn mask_first_n_bits(n: usize) -> Int {
    let mut o = [0u64; 16];
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

#[allow(dead_code)]
#[inline(always)]
pub fn zero() -> Int {
    U1024 { v: [0; 16] }
}

impl U1024 {
    pub fn from_words(words: [u64; 16]) -> Self {
        Self { v: words }
    }
}

use std::cmp::Ordering;

impl PartialEq for U1024 {
    fn eq(&self, other: &Self) -> bool {
        self.v == other.v
    }
}
impl Eq for U1024 {}

impl PartialOrd for U1024 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for U1024 {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare limbs from most-significant to least-significant
        for i in (0..16).rev() {
            if self.v[i] < other.v[i] { return Ordering::Less; }
            if self.v[i] > other.v[i] { return Ordering::Greater; }
        }
        Ordering::Equal
    }
}

use std::fmt;
impl fmt::Display for U1024 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.v.iter().all(|&x| x == 0) { return write!(f, "0"); }

        let mut n = *self;
        let mut digits = Vec::new();

        while n.v.iter().any(|&x| x != 0) {
            let mut remainder = 0u64;
            for i in (0..16).rev() {
                let value = ((remainder as u128) << 64) | (n.v[i] as u128);
                n.v[i] = (value / 10) as u64;
                remainder = (value % 10) as u64;
            }
            digits.push(remainder as u8);
        }

        for d in digits.iter().rev() { write!(f, "{}", d)?; }
        Ok(())
    }
}
impl fmt::Debug for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl BitAnd for U1024 {
    type Output = U1024;
    fn bitand(self, rhs: U1024) -> U1024 {
        let mut out = [0u64; 16];
        for i in 0..16 { out[i] = self.v[i] & rhs.v[i]; }
        U1024::from_words(out)
    }
}
impl BitOr for U1024 {
    type Output = U1024;
    fn bitor(self, rhs: U1024) -> U1024 {
        let mut out = [0u64; 16];
        for i in 0..16 { out[i] = self.v[i] | rhs.v[i]; }
        U1024::from_words(out)
    }
}
impl BitXor for U1024 {
    type Output = U1024;
    fn bitxor(self, rhs: U1024) -> U1024 {
        let mut out = [0u64; 16];
        for i in 0..16 { out[i] = self.v[i] ^ rhs.v[i]; }
        U1024::from_words(out)
    }
}
impl Not for U1024 {
    type Output = U1024;
    fn not(self) -> U1024 {
        let mut out = [0u64; 16];
        for i in 0..16 { out[i] = !self.v[i]; }
        U1024::from_words(out)
    }
}

/* Assign variants */
impl BitAndAssign for U1024 { fn bitand_assign(&mut self, rhs: U1024) { for i in 0..16 { self.v[i] &= rhs.v[i]; } } }
impl BitOrAssign for U1024 { fn bitor_assign(&mut self, rhs: U1024) { for i in 0..16 { self.v[i] |= rhs.v[i]; } } }
impl BitXorAssign for U1024 { fn bitxor_assign(&mut self, rhs: U1024) { for i in 0..16 { self.v[i] ^= rhs.v[i]; } } }

/* Shifts */
fn shl_words(src: &[u64; 16], n: usize) -> [u64; 16] {
    if n >= 1024 { return [0; 16]; }
    let word_shift = n / 64;
    let bit_shift = n % 64;
    let mut out = [0u64; 16];

    if bit_shift == 0 {
        for i in (word_shift..16).rev() { out[i] = src[i - word_shift]; }
        return out;
    }

    for i in (0..16).rev() {
        let src_idx = i as isize - word_shift as isize;
        if src_idx < 0 { out[i] = 0; } 
        else {
            let lo = src[src_idx as usize] << bit_shift;
            let hi = if src_idx as usize >= 1 { src[src_idx as usize - 1] >> (64 - bit_shift) } else { 0 };
            out[i] = lo | hi;
        }
    }
    out
}

fn shr_words(src: &[u64; 16], n: usize) -> [u64; 16] {

    let mut out = [0u64; 16];

    if n >= 1024 {
        return out;
    }

    let word_shift = n >> 6;
    let bit_shift = n & 63;  // bit_shift = n % 64

    if bit_shift == 0 {
        let len = 16 - word_shift;
        out[..len].copy_from_slice(&src[word_shift..]);
        return out;
    }

    let inv = 64 - bit_shift;

    let max = 16 - word_shift - 1;

    for i in 0..max {
        let j = i + word_shift;
        out[i] = (src[j] >> bit_shift) | (src[j + 1] << inv);
    }

    if max < 16 {
        out[max] = src[max + word_shift] >> bit_shift;
    }

    out
}

#[inline(always)]
fn shr_words_one(src: &[u64; 16]) -> [u64; 16] {

    let mut out = [0u64; 16];

    out[0] = (src[0] >> 1) | (src[1] << 63);
    out[1] = (src[1] >> 1) | (src[2] << 63);
    out[2] = (src[2] >> 1) | (src[3] << 63);
    out[3] = (src[3] >> 1) | (src[4] << 63);
    out[4] = (src[4] >> 1) | (src[5] << 63);
    out[5] = (src[5] >> 1) | (src[6] << 63);
    out[6] = (src[6] >> 1) | (src[7] << 63);
    out[7] = (src[7] >> 1) | (src[8] << 63);
    out[8] = (src[8] >> 1) | (src[9] << 63);
    out[9] = (src[9] >> 1) | (src[10] << 63);
    out[10] = (src[10] >> 1) | (src[11] << 63);
    out[11] = (src[11] >> 1) | (src[12] << 63);
    out[12] = (src[12] >> 1) | (src[13] << 63);
    out[13] = (src[13] >> 1) | (src[14] << 63);
    out[14] = (src[14] >> 1) | (src[15] << 63);
    out[15] = src[15] >> 1;

    out
}

#[inline(always)]
fn shl_words_one(src: &[u64; 16]) -> [u64; 16] {

    let mut out = [0u64; 16];

    out[0] = src[0] << 1;
    out[1] = (src[1] << 1) | (src[0] >> 63);
    out[2] = (src[2] << 1) | (src[1] >> 63);
    out[3] = (src[3] << 1) | (src[2] >> 63);
    out[4] = (src[4] << 1) | (src[3] >> 63);
    out[5] = (src[5] << 1) | (src[4] >> 63);
    out[6] = (src[6] << 1) | (src[5] >> 63);
    out[7] = (src[7] << 1) | (src[6] >> 63);
    out[8] = (src[8] << 1) | (src[7] >> 63);
    out[9] = (src[9] << 1) | (src[8] >> 63);
    out[10] = (src[10] << 1) | (src[9] >> 63);
    out[11] = (src[11] << 1) | (src[10] >> 63);
    out[12] = (src[12] << 1) | (src[11] >> 63);
    out[13] = (src[13] << 1) | (src[12] >> 63);
    out[14] = (src[14] << 1) | (src[13] >> 63);
    out[15] = (src[15] << 1) | (src[14] >> 63);

    out
}

#[inline(always)]
fn shr_words_two(src: &[u64; 16]) -> [u64; 16] {

    let mut out = [0u64; 16];

    out[0] = (src[0] >> 2) | (src[1] << 62);
    out[1] = (src[1] >> 2) | (src[2] << 62);
    out[2] = (src[2] >> 2) | (src[3] << 62);
    out[3] = (src[3] >> 2) | (src[4] << 62);
    out[4] = (src[4] >> 2) | (src[5] << 62);
    out[5] = (src[5] >> 2) | (src[6] << 62);
    out[6] = (src[6] >> 2) | (src[7] << 62);
    out[7] = (src[7] >> 2) | (src[8] << 62);
    out[8] = (src[8] >> 2) | (src[9] << 62);
    out[9] = (src[9] >> 2) | (src[10] << 62);
    out[10] = (src[10] >> 2) | (src[11] << 62);
    out[11] = (src[11] >> 2) | (src[12] << 62);
    out[12] = (src[12] >> 2) | (src[13] << 62);
    out[13] = (src[13] >> 2) | (src[14] << 62);
    out[14] = (src[14] >> 2) | (src[15] << 62);
    out[15] = src[15] >> 2;

    out
}

#[inline(always)]
fn shl_words_two(src: &[u64; 16]) -> [u64; 16] {

    let mut out = [0u64; 16];

    out[0] = src[0] << 2;
    out[1] = (src[1] << 2) | (src[0] >> 62);
    out[2] = (src[2] << 2) | (src[1] >> 62);
    out[3] = (src[3] << 2) | (src[2] >> 62);
    out[4] = (src[4] << 2) | (src[3] >> 62);
    out[5] = (src[5] << 2) | (src[4] >> 62);
    out[6] = (src[6] << 2) | (src[5] >> 62);
    out[7] = (src[7] << 2) | (src[6] >> 62);
    out[8] = (src[8] << 2) | (src[7] >> 62);
    out[9] = (src[9] << 2) | (src[8] >> 62);
    out[10] = (src[10] << 2) | (src[9] >> 62);
    out[11] = (src[11] << 2) | (src[10] >> 62);
    out[12] = (src[12] << 2) | (src[11] >> 62);
    out[13] = (src[13] << 2) | (src[12] >> 62);
    out[14] = (src[14] << 2) | (src[13] >> 62);
    out[15] = (src[15] << 2) | (src[14] >> 62);

    out
}

// #[target_feature(enable = "avx2")]
// unsafe fn shr_words_avx2(src: &[u64; 16], n: usize) -> [u64; 16] {
//     use std::arch::x86_64::*;
// 
//     let word_shift = n >> 6;
//     let bit_shift = (n & 63) as i32;
//     let inv = 64 - bit_shift;
// 
//     let mut out = [0u64; 16];
// 
//     for i in (0..16).step_by(4) {
//         let j = i + word_shift;
//         if j + 4 >= 17 { break; }
// 
//         let v0 = unsafe { _mm256_loadu_si256(src[j..].as_ptr() as *const __m256i) };
//         let v1 = unsafe { _mm256_loadu_si256(src[j+1..].as_ptr() as *const __m256i) };
// 
//         let lo = _mm256_srli_epi64(v0, const bit_shift);
//         let hi = _mm256_slli_epi64(v1, inv as *const i32);
// 
//         let r = _mm256_or_si256(lo, hi);
//         _mm256_storeu_si256(out[i..].as_mut_ptr() as *mut __m256i, r);
//     }
// 
//     out
// }

impl U1024 {
    pub fn shr1(&self) -> Self {
        U1024 { v: shr_words_one(&self.v) }
    }
    pub fn shr2(&self) -> Self {
        U1024 { v: shr_words_two(&self.v) }
    }
    pub fn shl1(&self) -> Self {
        U1024 { v: shl_words_one(&self.v) }
    }
    pub fn shl2(&self) -> Self {
        U1024 { v: shl_words_two(&self.v) }
    }
}


impl Shl<usize> for U1024 { type Output = U1024; fn shl(self, rhs: usize) -> U1024 { U1024::from_words(shl_words(&self.v, rhs)) } }
impl Shr<usize> for U1024 { type Output = U1024; fn shr(self, rhs: usize) -> U1024 { U1024::from_words(shr_words(&self.v, rhs)) } }
impl ShlAssign<usize> for U1024 { fn shl_assign(&mut self, rhs: usize) { self.v = shl_words(&self.v, rhs); } }
impl ShrAssign<usize> for U1024 { fn shr_assign(&mut self, rhs: usize) { self.v = shr_words(&self.v, rhs); } }

impl Shl<u32> for U1024 { type Output = U1024; fn shl(self, rhs: u32) -> U1024 { U1024::from_words(shl_words(&self.v, rhs as usize)) } }
impl Shr<u32> for U1024 { type Output = U1024; fn shr(self, rhs: u32) -> U1024 { U1024::from_words(shr_words(&self.v, rhs as usize)) } }
impl ShlAssign<u32> for U1024 { fn shl_assign(&mut self, rhs: u32) { self.v = shl_words(&self.v, rhs as usize); } }
impl ShrAssign<u32> for U1024 { fn shr_assign(&mut self, rhs: u32) { self.v = shr_words(&self.v, rhs as usize); } }

/* Reference convenience */
impl<'a> BitAnd for &'a U1024 { type Output = U1024; fn bitand(self, rhs: &'a U1024) -> U1024 { *self & *rhs } }
impl<'a> BitOr for &'a U1024 { type Output = U1024; fn bitor(self, rhs: &'a U1024) -> U1024 { *self | *rhs } }
impl<'a> BitXor for &'a U1024 { type Output = U1024; fn bitxor(self, rhs: &'a U1024) -> U1024 { *self ^ *rhs } }

impl U1024 {
    pub fn trailing_zeros(&self) -> u32 {
        for i in 0..16 { if self.v[i] != 0 { return (i as u32) * 64 + self.v[i].trailing_zeros(); } }
        1024
    }

    pub fn reverse_bits(&self) -> U1024 {
        let mut out = [0u64; 16];
        for i in 0..16 { out[i] = self.v[15 - i].reverse_bits(); }
        U1024::from_words(out)
    }

    pub fn min(self, other: U1024) -> U1024 {
        for i in (0..16).rev() {
            if self.v[i] < other.v[i] { return self; } 
            else if self.v[i] > other.v[i] { return other; }
        }
        self
    }

    pub fn ilog2(&self) -> u32 {
        for i in (0..16).rev() {
            if self.v[i] != 0 { return (i as u32) * 64 + 63 - self.v[i].leading_zeros(); }
        }
        panic!("ilog2 called on zero");
        0
    }
}
