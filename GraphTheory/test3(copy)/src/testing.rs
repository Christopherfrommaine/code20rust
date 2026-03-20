use std::collections::HashSet;

#[allow(unused_imports)]
use log::{debug, info};

use crate::solver::Solver;
use crate::timeit;

type F = Box<dyn Fn(Vec<usize>) -> usize + Send + Sync>;

fn test1() {timeit!({
    // // Rule 108
    let f: F = Box::new(|v| if vec![vec![1, 1, 0], vec![1, 0, 1], vec![0, 1, 1]].contains(&v) {1} else {0});
    let mut s = Solver::from_debruijn_parametric(3, f);

    // // Rule 108
    // let f: F = Box::new(|v| if vec![vec![1, 1, 0], vec![1, 0, 1], vec![0, 1, 1], vec![0, 1, 0]].contains(&v) {1} else {0});
    // let mut s = Solver::from_debruijn_parametric(3, f);
    
    // // Rule 30
    // let f: F = Box::new(|v| if vec![vec![1, 0, 0], vec![0, 1, 1], vec![0, 1, 0], vec![0, 0, 1]].contains(&v) {1} else {0});
    // let mut s = Solver::from_debruijn_parametric(3, f);
    
    // Code 20
    // let f: F = Box::new(|v: Vec<usize>| if {let o: usize = v.iter().sum(); o == 2 || o == 4} {1} else {0});
    // let mut s = Solver::from_debruijn_parametric(5, f);


    #[cfg(feature = "render")]
    s.fa.render_named_wl("sll");

    #[allow(unused)]
    let mut seen: HashSet<crate::fa::FA<usize>> = HashSet::new();

    for i in 1..50 {if timeit!({
        
        eprintln!("{i}");

        let sfa1 = timeit!({s.step()}, "stepping");
        #[cfg(feature = "render-all")]
        {sfa1.render_timestamped_wl("sfa1");}

        let sfa2 = timeit!({sfa1.to_usize_fa().bs_usize_to_dfa()}, "dfa conversion");
        #[cfg(feature = "render-all")]
        {sfa2.render_timestamped_wl("sfa2");}

        let sfa3 = timeit!({sfa2.to_usize_fa().usize_full_simplify()}, "simplifications");
        #[cfg(feature = "render-all")]
        {sfa3.render_timestamped_wl("sfa3");}

        s.fa = sfa3;

        #[cfg(feature = "render")]
        s.fa.render_named_wl(&format!("sll{i}"));

        // eprintln!("{i} | {:?}", s.without_null_loop().all_words(10000, 0, 20, 0));
        // println!("{},", format!("{:?}", s.without_null_loop().all_words(10000, 0, 20, 0)).replace("[", "{").replace("]", "}"));

        let mut o = false;

        if false && !seen.insert(s.fa.to_usize_fa().canonical_label()) {
            println!("Duplicate found after full step. Stopping...");
            o = true;
        }
        if s.fa.q.len() == 0 {
            o = true;
            println!("Empty q. Stopping...");
        }
        if s.fa.t.len() == 0 {
            o = true;
            println!("Empty t. Stopping...");
        }
        if o {info!("finished!");}
        o
    }, "full step") {break;} }
    println!("{},", format!("{:?}", s.fa.all_words(0, 0, 100, 0)).replace("[", "{").replace("]", "}"));

}, "full program");}


#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

pub fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    
    test1();
}