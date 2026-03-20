mod bruteforce;
mod satsolver;
mod finitestatemachine;

use crate::satsolver::filesatsolver;

fn main() {
    // filesolver::main();
    // filesolver::general_run_symmetric(5, 1);
    // filesolver::find_specific(10);
    // filesolver::main_symmetric();

    // for p in [1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 16, 18, 29] {
    //     filesolver::find_specific(p);
    // }

    // use rayon::premlude::*;
    // [1, 2, 3, 4, 5, 6, 10].into_par_iter().for_each(|p| {
    //     let mut w = 2;
    //     loop {
    //         if filesatsolver::general_run_all(w, p) {break;}
    //         w = (w as f64 * 1.5) as i32;
    //     }
    // });

    // let mut args: Vec<i32> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
    // if args.len() == 0 {
    //     args = (1..50).collect();
    // }
    // args.into_iter().for_each(|p| {
    //     filesatsolver::general_run_all(100, p, 0);
    // });

    // let p = 18;
    // let w = (50. + 16. * (0.1 * p as f64).exp()) as i32;

    // println!("Solving {w}, {p}");
    // filesatsolver::general_run_all(w, p, 0);

    // GENERATES THE COMMAND:
    // filesatsolver::general_run(500, 10, 0);


    // filesatsolver::run_cnf_command("filesolver/cnf_for_w30_p22".to_string(), 30, 22, 0);
    filesatsolver::general_run_all(30, 22, 0);
}