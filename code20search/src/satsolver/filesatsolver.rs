use cgrustplot::helper::file;

use crate::satsolver::satcreator::create_cnf;
use std::fs::File;
use std::io::Write;

const SOLVER_MULTITHREADING: bool = true;

fn export_cnf(clauses: &Vec<Vec<i32>>, filename: &str) {
    let mut file = File::create(filename).expect("Failed to create file.");

    let num_clauses = clauses.len();
    let num_vars = clauses.iter()
        .flat_map(|clause| clause.iter().map(|&lit| lit.abs()))
        .max()
        .unwrap_or(0);

    // DIMACS header
    let mut out = vec![format!("p cnf {} {}\n", num_vars, num_clauses)];

    // Write each clause: literals separated by spaces and ending with a 0.
    for clause in clauses {
        for &literal in clause {
            out.push(literal.to_string() + " ")
        }
        out.push("0\n".to_string());
    }

    write!(file, "{}", out.concat()).expect("Failed to write to file.");
}

pub fn run_cnf_command(filename: String, w: i32, p: i32, d: i32) -> bool {
    use std::process::Command;
    use std::os::unix::process::CommandExt; // For `before_exec`
    use nix::sys::prctl;

    let mut r = unsafe { Command::new("sh")
            .arg("-c")
            .arg(format!("./cryptominisat5 {} {filename}.cnf > {filename}_output.txt", if SOLVER_MULTITHREADING {"-t 16"} else {""}))
            .pre_exec(|| {
                prctl::set_pdeathsig(nix::sys::signal::Signal::SIGSTOP).expect("Failed to set parent death signal");
                Ok(())
            })
            .spawn()
            .expect("Failed to create process")
        };

    let r2 = r.wait();

    println!("Ran ({w}, {p}) with result: ({r:?}, {r2:?})");

    println!("parsing ({w}, {p})...");

    parse_file_output(&(filename + "_output.txt"), w, p, d)

}

fn parse_file_output(filename: &str, w: i32, p: i32, d: i32) -> bool {

    let string: String = std::fs::read_to_string(filename).expect(&format!("Could not read file: {}.", filename));

    let mut o: Vec<String> = Vec::new();

    // if filename.contains("symmetric") { return false; }  // manual for now

    string.split('\n').for_each(|s|
        if s.len() >= 3 && s.chars().nth(0) == Some('v') {
            o.push((&s[2..]).to_string())
        }
    );
    
    let res = o.iter().flat_map(|s| s.chars()).collect::<String>();

    let mut o: Vec<i32> = res.split_whitespace().filter_map(|s| s.parse().ok()).collect();

    if filename.contains("symmetric") {
        let orev: &Vec<i32> = &(o.iter().map(|x| *x).rev().collect());
        o.extend_from_slice(orev);
    }

    handle_result(o, if filename.contains("symmetric") {2 * w} else {w}, p, d)
}

fn handle_result(o: Vec<i32>, w: i32, p: i32, d: i32) -> bool {
    // println!("o ({w}, {p}): {o:?}");

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("total-file-output.txt")
        .expect("Couldn't create output file");

    if o.len() >= 5 {
        crate::satsolver::splrsatsolver::handle_sol(o.clone(), w, p);

        use crate::bruteforce::customuint::U256;
        let mut as_num: U256 = U256::from(0);
        for i in &o[1..(w as usize)] {
            as_num = as_num << 1;
            if i > &0 { as_num += U256::from(1); }
        }

        writeln!(file, "({w}, {p}, {}): {as_num:?}", d).expect("Couldn't write to output file");

        return true;

    } else {
        println!("Couldn't handle due to small solution");
    }
    
    return false;
}

pub fn general_run(width: i32, period: i32, diag: i32) -> bool {
    println!("creating ({width}, {period})...");
    let cnf = create_cnf(width, period, diag);
    let filename = format!("filesolver/cnf_for_w{width}_p{period}");

    println!("exporting ({width}, {period})...");
    export_cnf(&cnf, &(filename.clone() + ".cnf"));

    println!("running ({width}, {period})...");
    run_cnf_command(filename.clone(), width, period, diag)
}

#[allow(dead_code)]
pub fn general_run_all(width: i32, period: i32, diag: i32) -> bool {
    let mut iteration = 0;
    let mut cnf = create_cnf(width, period, diag);

    let mut final_output = false;

    loop {
        println!("creating ({width}, {period}, {diag}, {iteration})...");
        
        let filename = format!("filesolver/cnf_for_w{width}_p{period}_i{iteration}");
    
        println!("exporting ({width}, {period}, {diag}, {iteration})...");
        export_cnf(&cnf, &(filename.clone() + ".cnf"));
    
        println!("running ({width}, {period}, {diag}, {iteration})...");
        let res = run_cnf_command(filename.clone(), width, period, diag);
        
        if !res {break;}

        final_output = true;

        let string: String = std::fs::read_to_string(filename.clone() + "_output.txt").expect(&format!("Could not read file: {}.", filename));

        let mut o: Vec<String> = Vec::new();

        string.split('\n').for_each(|s|
            if s.len() >= 3 && s.chars().nth(0) == Some('v') {
                o.push((&s[2..]).to_string())
            }
        );
        
        let res: String = o.iter().flat_map(|s| s.chars()).collect();

        let output: Vec<i32> = res.split_whitespace().filter_map(|s| s.parse().ok()).collect();

        // handles elsewhere?????
        // handle_result(output.clone(), width, period, diag);

        let first_row = output[1..(width as usize + 1)].to_vec();
        cnf.push(first_row.iter().map(|x| -x).collect());

        let binary_row: Vec<bool> = first_row.iter().map(|d| d > &0).collect();
        let row_bytes: Vec<u8> = crate::satsolver::splrsatsolver::vec_bool_to_bytes(&binary_row);
        if row_bytes.len() < 4 * 8 {
            use crate::bruteforce::customuint::U256;

            let state = U256::from_big_endian(&row_bytes);

            let mut s = state;

            for _ in 0..(2 * period) {
                s = crate::bruteforce::bruteforce::step(s);

                if (s & U256::from(1)).is_zero() {continue;}

                let thing = crate::satsolver::splrsatsolver::u256tobits(s);

                cnf.push(
                    first_row
                    .iter()
                    .zip(thing.iter())
                    .map(|(num, neg)|
                    num * (if *neg == 1 {-1} else {1})
                ).collect());
            }

        }

        iteration += 1;
    }

    final_output
}

pub fn general_run_symmetric(width: i32, period: i32, diag: i32) -> bool {
    println!("creating s({width}, {period})...");
    let cnf = crate::satsolver::symmetricsatcreator::create_symmetric_cnf(width, period);
    let filename = format!("filesolver/cnf_for_w{width}_p{period}_symmetric");

    println!("exporting s({width}, {period})...");
    export_cnf(&cnf, &(filename.clone() + ".cnf"));

    println!("running s({width}, {period})...");
    run_cnf_command(filename.clone(), width, period, diag)
}

#[allow(dead_code)]
pub fn main() {
    vec![
        (010, 02),
        (100, 10),
        (150, 12),
        (200, 07),
        (200, 11),
        (200, 13),
        (200, 17),
    ].into_iter().for_each(|(w, p)| {general_run(w, p, 0);});

    (1..100).into_iter().for_each(|p| {general_run(15 * p, p, 0);});
}

#[allow(dead_code)]
pub fn main_symmetric() {
    vec![
        (010, 02),
        (100, 10),
        (150, 12),
        (200, 07),
        (200, 11),
        (200, 13),
        (200, 17),
    ].into_iter().for_each(|(w, p)| {general_run_symmetric(w / 2, p, 0);});

    (1..100).into_iter().for_each(|p| {general_run_symmetric(7 * p, p, 0);})
}

#[allow(dead_code)]
pub fn test() {
    for (w, p) in vec![
        (010, 01),
        (010, 02),
        (020, 04),
        (050, 03),
        (100, 06),
    ] {
        general_run(w, p, 0);
    }
}

#[allow(dead_code)]
pub fn find_specific(p: i32) {
    for w in [10, 100, 200, 300, 500, 700, 1000] {
        let o = general_run(w, p, 0);

        if o {break;}

        if p >= 9 && p % 2 == 1 && w >= 100 {break;}  // odd periods are really slow for some reason
    }
}

#[allow(dead_code)]
pub fn fast_large_width() {
    use rayon::prelude::*;

    for p in [1, 2, 3, 4, 5, 6, 7, 8, 10] {
        [10, 50, 100, 200, 500, 1000, 2000].into_par_iter().for_each(|w| {general_run(w, p, 0);});
    }
}