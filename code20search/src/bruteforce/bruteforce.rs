use rayon::prelude::*;  // Paralellization
use std::collections::HashSet;  // Hashing seen states
use cgrustplot::plots::array_plot::array_plot;  // Plotting automata
use std::{fs::OpenOptions, io::Write};  // Writing to file
use crate::bruteforce::customuint::U256;

type T = U256;
const TB: u32 = 256;

// Implements a code-20 step on a integer acting like a boolean list.
#[inline(always)]
pub fn step(init: T) -> T {
    // Bitshift to have the neighbors of each bit be (a, b, c, d, e)
    let a = init.rotate_right(2);
    let b = init.rotate_right(1);
    let c = init;
    let d = init.rotate_left(1);
    let e = init.rotate_left(2);

    // Bitwise definition of code 20
    (a | b | c | d | e) ^ (a ^ b ^ c ^ d ^ e)
}

#[inline(always)]
fn lifetimeinfinitewithoutput(init: T, max: u32) -> (bool, T) {
    let mut o = init;
    for _ in 0..max {
        if o == T::from(0) {return (false, o);}

        o = step(o);
    }
    return (true, o);
}

// Converts a u64 to a list of bits. Used for printing and debugging
fn u64tobits(n: T) -> Vec<u8> {
    let mut bits: Vec<u8> = Vec::with_capacity(TB as usize);
    for i in (0..TB).rev() {
        bits.push(((n >> i) & T::from(1)).as_u64() as u8);
    }
    bits
}

// Creates a bitmask between two positions
fn set_bits_range(j: T, jn: T) -> T {
    if j <= jn {
        // Create a contiguous mask from j to jn
        ((T::from(1) << (jn - j + 1)) - 1) << j
    } else {
        // Wrap-around case: Set bits from j to 15 and from 0 to jn
        let high_mask = (!T::from(0)) << j;
        let low_mask = (T::from(1) << (jn + 1)) - 1;
        high_mask | low_mask
    }
}

// Tests for multiple patterns in a single automata
fn test_duplicates(next_hundred: &[T], h: &mut HashSet<T>) -> bool {
    // The way this works is by splitting across empty columns and checking for uniqueness on each side of the split

    // Find split positions (i.e. empty columns)
    const MAX_SEP: u32 = 3;
    let mut split_pos: Vec<T> = Vec::new();
    let mut num_split = 0;
    for g in 0..TB {
        if next_hundred.iter().all(|x| *x & (T::from(1) << g) == T::from(0)) {
            num_split += 1;
            if num_split == MAX_SEP {
                split_pos.push(T::from(g));
            }
        } else {
            num_split = 0;
        }
    }

    // Check if each side of the split is unique
    let mut only = true;
    for j in 0..split_pos.len() {
        let jn = (j + 1) % split_pos.len();
        let bm = set_bits_range(split_pos[j], split_pos[jn]);

        for ind in 0..next_hundred.len() {
            let mn = bm & next_hundred[ind];
            if mn != T::from(0) && h.contains(&(mn >> mn.trailing_zeros())) {
                only = false;
                break;
            }
        }

        if !only {break;}
    }

    only
}

fn format_u256_as_binary(value: U256) -> String {
    // Convert the U256 value to a binary string
    let mut binary_string = String::new();

    // Iterate through each bit position in the U256 (256 bits in total)
    for i in (0..256).rev() {
        // Check if the bit at the current position is set (1) or not (0)
        if value & (U256::from(1) << i) != U256::from(0) {
            binary_string.push('1');
        } else {
            binary_string.push('0');
        }
    }

    binary_string
}

#[allow(dead_code)]
pub fn main() {
    // Setup for the output file
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("output.txt")
        .expect("Failed to create output file.");

    // Stores previously seen states
    let mut h: HashSet<T> = HashSet::new();

    // Batching for paralellization
    let batch_size: u32 = 1_000_000;
    let mut bn: u32 = 0; // current batch number

    loop {
        // Generate list of possible initialization states
        let v: Vec<(T, T)> = (0..batch_size).into_par_iter()
            .map(|i| T::from(i) + batch_size * bn + 1)

            .flat_map(|i| {
                // Only search for symmetric solutions
                let o1 = i.reverse_bits() >> i.leading_zeros() | i << (1 + i.ilog2());
                let o2 = i.reverse_bits() >> i.leading_zeros() | i << i.ilog2();
                [o1, o2]
            })
            
            .filter_map(|i| {

                // Filter for nonhalting (remove all automata that halt)
                let (lif, mut s) = lifetimeinfinitewithoutput(i << (TB / 2), 500);
                if !lif {return None;}
                
                // Filter for uniqueness over 10 states
                for _ in 0..10 {
                    if h.contains(&(s >> s.trailing_zeros())) {return None;}
                    s = step(s);
                }

                Some((i, s))
            }).collect();
        
        // For non-halting, non-seen states:
        for (i, mut s) in v {
            // Generate 100 states
            let mut next_hundred = [T::from(0); 100];
            for n in 0..100 {
                s = step(s);
                next_hundred[n] = s;
            }

            // Check if any have been seen
            let not_unique = next_hundred.iter().any(|x| h.contains(&(x >> x.trailing_zeros())));
            if not_unique {continue;}

            // Check for more than one duplicated patterns
            let mut only = test_duplicates(&next_hundred, &mut h);

            // Check for diagonal repeating patterns
            for offset in 0..90 {
                if !only {break;}
                
                // I hate this, but it makes errors much more rare
                let text = format_u256_as_binary(next_hundred[offset]);

                let cs= vec!["10010111", "101011011", "10111101", "1000010011111001111100100001", "110110101", "1001111011", "1101111001", "11101101110011111001110110111", "10010001011011110111", 
 "10010101001010110111", "111000101110101000111111", "111111000101011101000111", "1011000000011011111111101100000001101", "10100111101010100100101010111100101", 
 "1110001111011101001001001011101111000111", "10100110001011110110111100011001100100001101"];
                only = only && !cs.into_iter().any(|string| text.contains(&("000".to_string() + string + "000")));
                only = only && test_duplicates(&(next_hundred[offset..(offset + 5)].iter().map(|x| x >> x.trailing_zeros()).collect::<Vec<T>>()), &mut h);
            }


            if only {
                for _ in 0..10_000 {
                    s = step(s);
                    if s.is_zero() {only = false; break;}
                }
            }
            
            // Update hash list with seen states
            for f in 0..TB {
                next_hundred.iter().for_each(|x| {
                    let xr = x.rotate_right(f);
                    h.insert(xr >> xr.trailing_zeros());
                    h.insert(xr.reverse_bits() >> xr.reverse_bits().trailing_zeros());
                });
            }

            // If the state is unique and only contains one pattern, write it to a file.
            if only {
                writeln!(file, "{},", i).expect("");
                
                println!("-----\nNew Found! {}:", i);
                array_plot(&next_hundred.iter().map(|x| u64tobits(*x)).collect()).set_axes(false).print();
                println!("-----")
            }
        }

        println!("Batch {bn}");

        bn += 1;
    }
}
