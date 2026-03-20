// Leter, I may need to change this to a custom u256 or u512 type,
// so implement neccesary things here.

use std::u128;


#[allow(dead_code)]
pub type Int = u128;


#[allow(dead_code)]
pub const BITS: usize = 128;


#[allow(dead_code)]
pub fn to_u8(n: Int) -> u8 {
    n as u8
}

#[allow(dead_code)]
pub fn from_u128(n: u128) -> Int {
    n
}

#[allow(dead_code)]
pub fn to_u128(n: Int) -> u128 {
    n
}

#[allow(dead_code)]
pub fn mask_first_n_bits(n: usize) -> Int {
    (1 << n) - 1
}


#[allow(dead_code)]
#[inline(always)]
pub fn one() -> Int {
    1
}


#[allow(dead_code)]
#[inline(always)]
pub fn zero() -> Int {
    0
}