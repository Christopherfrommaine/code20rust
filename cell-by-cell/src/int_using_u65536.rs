use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr, ShrAssign};
use std::cmp::Ordering;
use std::fmt;

#[derive(Clone, Copy)]
pub struct U65536 {
    v: [u64; 1024],
}

impl Default for U65536 {
    fn default() -> Self {
        U65536 { v: [0u64; 1024] }
    }
}


#[allow(dead_code)]
pub type Int = U65536;

#[allow(dead_code)]
pub const BITS: usize = 65536;

#[allow(dead_code)]
pub fn to_u8(n: Int) -> u8 {
    n.v[0] as u8
}

#[allow(dead_code)]
#[inline(always)]
pub fn one() -> Int {
    let mut v = [0u64; 1024];
    v[0] = 1;
    U65536 { v }
}

#[allow(dead_code)]
pub fn zero() -> Int {
    U65536 { v: [0; 1024] }
}

#[allow(dead_code)]
pub fn from_u128(n: u128) -> Int {
    let mut v = [0u64; 1024];
    v[0] = n as u64;
    v[1] = (n >> 64) as u64;
    U65536 { v }
}

#[allow(dead_code)]
pub fn to_u128(n: Int) -> u128 {
    debug_assert!(n.v[2..].iter().all(|&i| i == 0));
    n.v[0] as u128 + ((n.v[1] as u128) << 64)
}

#[allow(dead_code)]
pub fn mask_first_n_bits(n: usize) -> Int {
    let mut o = [0u64; 1024];
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

impl U65536 {
    pub fn from_words(words: [u64; 1024]) -> Self {
        Self { v: words }
    }

    pub fn trailing_zeros(&self) -> u32 {
        for i in 0..1024 {
            if self.v[i] != 0 {
                return (i as u32) * 64 + self.v[i].trailing_zeros();
            }
        }
        65536
    }

    pub fn reverse_bits(&self) -> Self {
        let mut out = [0u64; 1024];
        for i in 0..1024 {
            out[i] = self.v[1023 - i].reverse_bits();
        }
        U65536::from_words(out)
    }

    pub fn min(self, other: Self) -> Self {
        for i in (0..1024).rev() {
            if self.v[i] < other.v[i] { return self; }
            else if self.v[i] > other.v[i] { return other; }
        }
        self
    }

    pub fn ilog2(&self) -> u32 {
        for i in (0..1024).rev() {
            if self.v[i] != 0 {
                return (i as u32) * 64 + 63 - self.v[i].leading_zeros();
            }
        }
        panic!("ilog2 called on zero");
    }
}

// Implement comparison traits
impl PartialEq for U65536 { fn eq(&self, other: &Self) -> bool { self.v == other.v } }
impl Eq for U65536 {}
impl PartialOrd for U65536 { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) } }
impl Ord for U65536 {
    fn cmp(&self, other: &Self) -> Ordering {
        for i in (0..1024).rev() {
            if self.v[i] < other.v[i] { return Ordering::Less; }
            if self.v[i] > other.v[i] { return Ordering::Greater; }
        }
        Ordering::Equal
    }
}

// Implement Display & Debug
impl fmt::Display for U65536 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.v.iter().all(|&x| x == 0) { return write!(f, "0"); }

        let mut n = *self;
        let mut digits = Vec::new();

        while n.v.iter().any(|&x| x != 0) {
            let mut remainder = 0u64;
            for i in (0..1024).rev() {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { fmt::Display::fmt(self, f) }
}

// Implement bitwise operations
macro_rules! impl_bitwise {
    ($ty:ty, $len:expr) => {
        impl BitAnd for $ty {
            type Output = $ty;
            fn bitand(self, rhs: $ty) -> $ty {
                let mut out = [0u64; $len];
                for i in 0..$len { out[i] = self.v[i] & rhs.v[i]; }
                <$ty>::from_words(out)
            }
        }
        impl BitOr for $ty {
            type Output = $ty;
            fn bitor(self, rhs: $ty) -> $ty {
                let mut out = [0u64; $len];
                for i in 0..$len { out[i] = self.v[i] | rhs.v[i]; }
                <$ty>::from_words(out)
            }
        }
        impl BitXor for $ty {
            type Output = $ty;
            fn bitxor(self, rhs: $ty) -> $ty {
                let mut out = [0u64; $len];
                for i in 0..$len { out[i] = self.v[i] ^ rhs.v[i]; }
                <$ty>::from_words(out)
            }
        }
        impl Not for $ty {
            type Output = $ty;
            fn not(self) -> $ty {
                let mut out = [0u64; $len];
                for i in 0..$len { out[i] = !self.v[i]; }
                <$ty>::from_words(out)
            }
        }

        impl BitAndAssign for $ty { fn bitand_assign(&mut self, rhs: $ty) { for i in 0..$len { self.v[i] &= rhs.v[i]; } } }
        impl BitOrAssign for $ty { fn bitor_assign(&mut self, rhs: $ty) { for i in 0..$len { self.v[i] |= rhs.v[i]; } } }
        impl BitXorAssign for $ty { fn bitxor_assign(&mut self, rhs: $ty) { for i in 0..$len { self.v[i] ^= rhs.v[i]; } } }
    };
}

impl_bitwise!(U65536, 1024);

// Shift helpers
fn shl_words(src: &[u64; 1024], n: usize) -> [u64; 1024] {
    if n >= 65536 { return [0; 1024]; }
    let word_shift = n / 64;
    let bit_shift = n % 64;
    let mut out = [0u64; 1024];

    if bit_shift == 0 {
        for i in (word_shift..1024).rev() { out[i] = src[i - word_shift]; }
        return out;
    }

    for i in (0..1024).rev() {
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

fn shr_words(src: &[u64; 1024], n: usize) -> [u64; 1024] {
    if n >= 65536 { return [0; 1024]; }
    let word_shift = n / 64;
    let bit_shift = n % 64;
    let mut out = [0u64; 1024];

    if bit_shift == 0 {
        for i in 0..(1024 - word_shift) { out[i] = src[i + word_shift]; }
        return out;
    }

    for i in 0..1024 {
        let src_idx = i + word_shift;
        if src_idx >= 1024 { out[i] = 0; }
        else {
            let lo = src[src_idx] >> bit_shift;
            let hi = if src_idx + 1 < 1024 { src[src_idx + 1] << (64 - bit_shift) } else { 0 };
            out[i] = lo | hi;
        }
    }
    out
}

// Shift traits
impl Shl<usize> for U65536 { type Output = Self; fn shl(self, rhs: usize) -> Self { Self::from_words(shl_words(&self.v, rhs)) } }
impl Shr<usize> for U65536 { type Output = Self; fn shr(self, rhs: usize) -> Self { Self::from_words(shr_words(&self.v, rhs)) } }
impl ShlAssign<usize> for U65536 { fn shl_assign(&mut self, rhs: usize) { self.v = shl_words(&self.v, rhs); } }
impl ShrAssign<usize> for U65536 { fn shr_assign(&mut self, rhs: usize) { self.v = shr_words(&self.v, rhs); } }

impl Shl<u32> for U65536 { type Output = Self; fn shl(self, rhs: u32) -> Self { Self::from_words(shl_words(&self.v, rhs as usize)) } }
impl Shr<u32> for U65536 { type Output = Self; fn shr(self, rhs: u32) -> Self { Self::from_words(shr_words(&self.v, rhs as usize)) } }
impl ShlAssign<u32> for U65536 { fn shl_assign(&mut self, rhs: u32) { self.v = shl_words(&self.v, rhs as usize); } }
impl ShrAssign<u32> for U65536 { fn shr_assign(&mut self, rhs: u32) { self.v = shr_words(&self.v, rhs as usize); } }
