use rayon::{prelude::*, vec};

fn step(bv: &Vec<u8>) -> Vec<u8> {
    let bvc = |i: i32| if 0 <= i && i < bv.len() as i32 {bv[i as usize]} else {0};

    (0..bv.len() as i32).map(|i|
        (bvc(i - 2) ^ bvc(i - 1) ^ bvc(i) ^ bvc(i + 1) ^ bvc(i + 2)) ^ (bvc(i - 2) | bvc(i - 1) | bvc(i) | bvc(i + 1) | bvc(i + 2))
    ).collect()
}

fn steps(bv: Vec<u8>, t: u32) -> Vec<u8> {
    let mut o = bv;
    for _ in 0..t {
        o = step(&o);
    }
    o.to_vec()
}

pub fn stepsall(bv: Vec<u8>, t: u32) -> Vec<Vec<u8>> {
    let mut o = vec![bv];
    (0..t).for_each(|_| o.push(step(&o[o.len() - 1])));
    o
}

pub fn vec_to_u64(bits: Vec<u8>) -> u64 {
    let mut result = 0u64;
    for &bit in bits.iter().rev() {
        result = (result << 1) | (bit as u64);
    }
    result
}

pub fn finitestatemachine(p: u32, max_iter: i32, max_solutions: i32) -> Vec<Vec<u8>> {
    let mut states: Vec<Vec<u8>> = vec![vec![0; 4 * p as usize + 1]];
    let mut depth = 0;

    let mut prevlen = 0;

    loop {
        if !(depth < 2 * p) {
            print!("");
        }
        
        let stn: Vec<u64> = states.iter().map(|bv| vec_to_u64(bv.to_vec())  >> (4 * p + 1)).collect();
        println!("Beginning stn:{stn:?}");

        // DeBruijn Shift
        states.iter_mut().for_each(|bv| bv.push(0));
        states.extend_from_slice(&(states.clone().into_iter().map(|bv| {let mut o = bv; let olen = o.len() - 1; o[olen] = 1; o}).collect::<Vec<Vec<u8>>>()));

        let stn: Vec<u64> = states.iter().map(|bv| vec_to_u64(bv.to_vec())  >> (4 * p + 1)).collect();
        println!("DeBruijn stn:{stn:?}");

        // Center Invariance
        let centerbitpos = (depth + 2 * p) as usize + 1;

        println!("Center position: {centerbitpos}");
        if depth < 2 * p {
            states = states.into_iter().filter(|s| {
                let o = steps(s.clone(), p);
                o[centerbitpos] == 0
            }).collect();
        } else {
            states = states.into_iter().filter(|s| {
                let o = steps(s.clone(), p);
                o[centerbitpos] == s[centerbitpos]
            }).collect();
        }
        let stn: Vec<u64> = states.iter().map(|bv| vec_to_u64(bv.to_vec())  >> (4 * p + 1)).collect();
        println!("Center Invariant stn:{stn:?}");

        
        states = states.into_iter().filter(|s| {
            let sc = s.clone();
            let sn = vec_to_u64(s.clone());
            let snn = sn >> (4 * p + 1);

            if true || snn == 17 {
                print!("HERE DOES THE BREAKING THING");
            }

            let o: Vec<Vec<u8>> = stepsall(s.clone(), p - 1);

            if depth < 2 * p {return true;}

            let od: Vec<Vec<u8>> = o.into_iter().map(|r| r[..centerbitpos].to_vec()).collect();

            // Subperiod Constraint
            // if od.iter().skip(1).any(|r| r == &od[0]) {return false;}

            // Distinst Constraint
            let owidth = od[0].len();
            let oheight = p as usize;
            let combined: Vec<u8> = (0..owidth).map(|i| (0..oheight).map(|j| od[j][i]).fold(0, |a, b| a | b)).collect();

            let left = combined.iter().position(|x| *x != 0);
            let right = combined.iter().rev().position(|x| *x != 0);

            if left.is_none() || right.is_none() {
                return false;  // s == 0
            } else {
                let l = left.unwrap();
                let r = od[0].len() - 1 - right.unwrap();

                // if (l..(r - 1)).any(|i| s[i] + s[i + 1] + s[i + 2] == 0) {return false;}
                let mut any = false;
                for i in l..(r - 1) {
                    any = any || od[0][i] + od[0][i + 1] + od[0][i + 2] == 0;
                }
                if any {return false;}

            }

            true
        }).collect();

        let stn: Vec<u64> = states.iter().map(|bv| vec_to_u64(bv.to_vec())  >> (4 * p + 1)).collect();
        println!("Other Constrained stn:{stn:?}");

        // End
        if states.len() == prevlen || states.len() == 0 {println!("All solutions found!"); break;}
        prevlen = states.len();

        depth += 1;
        println!("{}", states.len());

        if depth as i32 == max_iter {println!("Stopping due to too many iters."); break;}
        if max_solutions >= 0 && (states.len() as i32) > max_solutions {println!("Stopping due to too many solutions."); break;}
    }

    states


}