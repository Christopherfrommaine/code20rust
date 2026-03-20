
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitSet {
    l: usize,
    v: Vec<u64>,
}

use std::hash::{Hash, Hasher};
impl Hash for BitSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut h: u64 = 0xcbf29ce484222325; // FNV offset basis
        for &w in &self.v {
            h ^= w;
            h = h.wrapping_mul(0x100000001b3); // FNV prime
        }
        // Final mix
        h ^= h >> 32;
        state.write_u64(h);
    }
}

impl BitSet {

    pub fn contains(&self, i: usize) -> bool {
        debug_assert!(i < self.l);

        let i1 = i >> 6;  // i1 = i / 64
        let i2 = i & 63;
        let i2_mask = 1u64 << i2;

        self.v[i1] & i2_mask != 0
    }

    pub fn insert(&mut self, i: usize) -> bool {
        debug_assert!(i < self.l);

        let i1 = i >> 6;  // i1 = i / 64
        let i2 = i & 63;
        let i2_mask = 1u64 << i2;

        let o = self.v[i1] & i2_mask != 0;

        self.v[i1] = self.v[i1] | i2_mask;

        o
    }

    pub fn from_vec(v: &[usize], l: usize) -> Self {

        let wl = (l + 63) / 64;

        let mut bs = BitSet {
            l,
            v: vec![0; wl],
        };

        for i in v {
            debug_assert!(*i < l);

            bs.v[i >> 6] |= 1u64 << (i & 63);
        }

        bs
    }

    pub fn with_capacity(l: usize) -> Self {
        let wl = (l + 63) / 64;

        BitSet {
            l,
            v: vec![0; wl],
        }
    }

    pub fn extend(&mut self, v: &[usize]) {

        for i in v {
            self.v[i >> 6] |= 1u64 << (i & 63);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.v.iter().all(|w| *w == 0)
    }

    pub fn clear(&mut self) {
        for w in &mut self.v { *w = 0; }
    }

    pub fn or_inplace(&mut self, other: &BitSet) {
        for (a, b) in self.v.iter_mut().zip(&other.v) { *a |= *b; }
    }

    pub fn iter(&self) -> BitSetIter<'_> {
        BitSetIter {
            bitset: self,
            word_index: 0,
            current_word: self.v.get(0).copied().unwrap_or(0),
            base_index: 0,
        }
    }
}





impl<'a> IntoIterator for &'a BitSet {
    type Item = usize;
    type IntoIter = BitSetIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BitSetIter {
            bitset: self,
            word_index: 0,
            current_word: if self.v.is_empty() { 0 } else { self.v[0] },
            base_index: 0,
        }
    }
}

pub struct BitSetIter<'a> {
    bitset: &'a BitSet,
    word_index: usize,
    current_word: u64,
    base_index: usize,
}

impl<'a> Iterator for BitSetIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.word_index < self.bitset.v.len() {
            if self.current_word != 0 {
                let tz = self.current_word.trailing_zeros() as usize;
                self.current_word &= !(1 << tz);
                return Some(self.base_index + tz);
            } else {
                self.word_index += 1;
                if self.word_index < self.bitset.v.len() {
                    self.current_word = self.bitset.v[self.word_index];
                    self.base_index = self.word_index * 64;
                }
            }
        }
        None
    }
}
