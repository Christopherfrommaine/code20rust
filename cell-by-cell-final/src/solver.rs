use crate::handle_solution::*;
use crate::int::*;

#[inline(always)]
pub fn code20(n: Int) -> Int {
    unsafe {
        let a = n.unchecked_shl(1);
        let b = n.unchecked_shl(2);
        let c = n;
        let d = n.unchecked_shr(1);
        let e = n.unchecked_shr(2);
        (a ^ b ^ c ^ d ^ e) ^ (a | b | c | d | e)
    }
}

#[inline(always)]
fn gap_length_less_than(mut n: Int, max: u32) -> bool {
    if n == zero() {return true;}
    unsafe {
        n = n.unchecked_shr(n.trailing_zeros());

        while n != zero() {
            let tz = n.trailing_zeros();
            if tz > max { return false; }
            n = n.unchecked_shr(tz + 1);
        }
    }
    true
}

pub fn solve(p: usize) {
    println!("Starting solve with p{p}, s0");
    log_to_file(format!("(p{p}, s0):\n"));

    let len_i = 2 * p;
    let n_i = one() << len_i;

    let nodes_searched = solve_dfs(n_i, len_i, p);

    println!("Nodes Searched: {}", nodes_searched);
}

pub fn solve_dfs(n: Int, len: usize, p: usize) -> u64 {
    // len is the position (zero-indexed from right to left) of the first possible 1

    // Depth Exceeded
    if len > BITS - 2 * p - 5 {
        if p != 8 {eprintln!("DEPTH LIMIT REACHED\n{n}")};
        // std::process::exit(1);
        return 1;
    }

    // Run the automata
    let mut collected = zero();
    let mut o = n;
    for _ in 0..p {
        o = code20(o);
        collected |= o;
    }

    // Unchangable output bits
    let mask = mask_first_bits(len - 2 * p + 1);
    
    // Check Periodicity
    if n & mask != o & mask {return 1;}

    // Check Gaps (for concatonated solutions)
    if !gap_length_less_than(collected & mask, 2) {return 1;}

    // // I'd rather just ignore this honestly since it only appears for p=8
    // // Tilability check (for infinitely repeatable patterns)
    // let mut n_count_ones;  // deffered for speed
    // for pattern_length in 1..((len + 1) / 2) {
    //     let pattern = unsafe { n & (mask_first_bits(pattern_length).unchecked_shl((len - pattern_length) as u32 + 1)) };
    // 
    //     if pattern == zero() {continue;}
    //     
    //     let mut rep = zero();
    //     for i in 0..((len / pattern_length) + 1) {
    //         rep |= unsafe { pattern.unchecked_shr((pattern_length * i) as u32) }
    //     }
    // 
    //     n_count_ones = n.count_ones();
    //     if (n ^ rep).count_ones() + 10 < n_count_ones / 2 {
    //         return 1;
    //     }
    // }

    // Check for Solution
    if o == n {
        // More expensive full run
        let mut current = o;

        // keeps track of smallest value including reflections
        let mut minorg = o;
        let mut minrev = o.reverse_bits();

        for _count in 1..p {
            current = code20(current);
            minorg = minorg.min(current);
            minrev = minrev.min(current.reverse_bits());

            // No Subperiodicity
            if current == o {return 1;}
        }

        // Minimum integer acheived over period
        let minorg = minorg >> minorg.trailing_zeros();
        let minrev = minrev >> minrev.trailing_zeros();
        let min = if minorg < minrev {minorg} else {minrev};

        // Overwrites may happen! But this ensures that solutions are found faster, if they exist
        handle_found_solution(min, p);

        return 1;
    }

    // No checks have eliminated cantidate. Continue search.
    let new_len = len + 1;

    let mut nodes_searched = 1;

    if len > 2 * p + 10  {
        // Basic solve at large depths
        nodes_searched += solve_dfs(n, new_len, p);
        nodes_searched += solve_dfs(n | unsafe { one().unchecked_shl(new_len as u32) }, new_len, p);
    } else {
        // Paralelleized solve at top-level nodes
        let mut ns1 = 0;
        let mut ns2 = 0;
        rayon::join(
            || ns1 = solve_dfs(n, new_len, p),
        || ns2 = solve_dfs(n | unsafe { one().unchecked_shl(new_len as u32) }, new_len, p),
        );

        nodes_searched += ns1 + ns2;
    }

    nodes_searched
    
}