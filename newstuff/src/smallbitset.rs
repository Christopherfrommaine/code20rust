use std::hash::{Hash, Hasher};

const WORDS: usize = 8; // supports up to 256 NFA states; adjust as needed

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct SmallBitSet {
    pub bits: [u64; WORDS],
}

impl Hash for SmallBitSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bits.hash(state);
    }
}

impl SmallBitSet {
    pub fn new() -> Self {
        SmallBitSet { bits: [0; WORDS] }
    }

    pub fn from_range(n: usize) -> Self {
        let mut s = Self::new();
        for i in 0..n {
            s.set(i);
        }
        s
    }

    #[inline]
    pub fn set(&mut self, i: usize) {
        self.bits[i / 64] |= 1u64 << (i % 64);
    }

    #[inline]
    pub fn contains(&self, i: usize) -> bool {
        self.bits[i / 64] & (1u64 << (i % 64)) != 0
    }

    pub fn is_empty(&self) -> bool {
        self.bits.iter().all(|&w| w == 0)
    }

    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.bits.iter().enumerate().flat_map(|(wi, &word)| {
            let base = wi * 64;
            (0..64).filter_map(move |bi| {
                if word & (1u64 << bi) != 0 {
                    Some(base + bi)
                } else {
                    None
                }
            })
        })
    }

    pub fn to_vec(&self) -> Vec<usize> {
        self.iter().collect()
    }
}