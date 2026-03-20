use uint::construct_uint;

type T = U256;

construct_uint! {
    pub struct U256(4); // 4 * 64-bit = 256-bit integer
}

impl T {
    pub fn rotate_one_left(self) -> Self {
        self << 1 | self >> 255
    }

    pub fn rotate_two_left(self) -> Self {
        self << 2 | self >> 254
    }

    pub fn rotate_one_right(self) -> Self {
        self >> 1 | self << 255
    }

    pub fn rotate_two_right(self) -> Self {
        self >>1 | self << 254
    }

    /// Rotate left by `n` bits
    pub fn rotate_left(self, n: u32) -> Self {
        let n = n % 256;
        (self << n) | (self >> (256 - n))
    }

    /// Rotate right by `n` bits
    pub fn rotate_right(self, n: u32) -> Self {
        let n = n % 256;
        (self >> n) | (self << (256 - n))
    }

    pub fn reverse_bits(self) -> Self {
        let mut limbs = self.0;
        
        // Reverse bits in each 64-bit limb
        for limb in &mut limbs {
            *limb = limb.reverse_bits();
        }
        
        // Swap limbs to reflect bit order reversal
        limbs.reverse();
        
        Self(limbs)
    }

    pub fn ilog2(self) -> u32 {
        for (i, &limb) in self.0.iter().enumerate().rev() {
            if limb != 0 {
                // The log2 of a non-zero limb is calculated.
                return (i as u32 * 64) + limb.ilog2();
            }
        }
        0
    }
}



#[test]
fn test_bit_rotations() {
    let x = T::from(0b1011);
    let rotated_left = x.rotate_left(2);
    let rotated_right = x.rotate_right(2);

    println!("Original:       {:}", x);
    println!("Rotate Left:  {:}", rotated_left);
    println!("Rotate Right: {:}", rotated_right);
}
