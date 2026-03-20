use splr::*;
use cgrustplot::plots::array_plot;
use rayon::prelude::*;
use crate::bruteforce::customuint::U256;
use std::panic::catch_unwind;

use crate::satsolver::satcreator::*;

#[allow(dead_code)]
pub fn main() {
    (1..10).into_par_iter().for_each(|i| {
        let mut w = 8;
        loop {
            let o = run_thing(w, i);

            if o {break;}

            w += 8;

            if w > 200 {break;}
        }
    });

    (1..100).into_par_iter().for_each(|i| {
        let mut w = 3;
        loop {
            let o = run_thing(1 << w, i);

            if o {break;}

            w += 1;

            if w >= 8 {break;}
        }
    });

    vec![50, 100, 200, 500].into_par_iter().for_each(|w| {run_thing(w, 12);});
}

pub fn run_thing(width: i32, period: i32) -> bool {

    let non_taut = create_cnf(width, period, 0);

    let output = catch_unwind(|| Certificate::try_from(non_taut));

    if output.is_err() {return true;}

    let output2 = output.expect("whoops");

    match output2 {
        Ok(Certificate::SAT(ans)) => {handle_sol(ans, width, period); return true;},
        Ok(Certificate::UNSAT) => println!("s UNSATISFIABLE for period {period} and width {width}"),
        Err(e) => println!("ERROR: s UNKNOWN p: {period} w: {width}; {}", e),
    }

    false
}

fn step256(init: U256) -> U256 {
    // Bitshift to have the neighbors of each bit be (a, b, c, d, e)
    let a = init.rotate_right(2);
    let b = init.rotate_right(1);
    let c = init;
    let d = init.rotate_left(1);
    let e = init.rotate_left(2);

    // Bitwise definition of code 20
    (a | b | c | d | e) ^ (a ^ b ^ c ^ d ^ e)
}

pub fn u256tobits(n: U256) -> Vec<u8> {
    let mut bits: Vec<u8> = Vec::with_capacity(256);
    for i in (0..256).rev() {
        bits.push(((n >> i) & U256::from(1)).as_u64() as u8);
    }
    bits
}

pub fn vec_bool_to_bytes(bits: &[bool]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((bits.len() + 7) / 8);
    for chunk in bits.chunks(8) {
        let mut byte = 0u8;
        for &bit in chunk {
            byte = (byte << 1) | (bit as u8);
        }
        // Shift left to align MSB if chunk is less than 8 bits
        byte <<= 8 - chunk.len();
        bytes.push(byte);
    }
    bytes
}

pub fn handle_sol(ans: Vec<i32>, width: i32, period: i32) {
    println!("Solution Found for period {period}!");

    let first_row = &ans[1..(width as usize + 1)];
    let binary_row: Vec<bool> = first_row.iter().map(|d| d > &0).collect();
    let row_bytes: Vec<u8> = vec_bool_to_bytes(&binary_row);
    if row_bytes.len() < 4 * 8 {
        let state = U256::from_big_endian(&row_bytes);



        println!("State: {state}");

        let mut s = state;
        
        array_plot::array_plot(&(0..(5 * period)).map(|_| {let temp = s; s = step256(s); temp}).map(|x| {let mut o = u256tobits(x); o.reverse(); o}).collect()).set_axes(false).print();

    } else {
        // println!("State (as list): {binary_row:?}");
    }
    
}
