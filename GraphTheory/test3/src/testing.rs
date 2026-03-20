use std::collections::HashSet;
use log::*;
use rustc_hash::FxHashSet;
use crate::cellularautomation::code20step;
use crate::solver::Solver;
use crate::timeit;

// A Cellular Automaton Rule
type F = Box<dyn Fn(Vec<usize>) -> usize + Send + Sync>;

fn nest(f: F, n: u32, v: Vec<usize>) -> usize {

    if n == 1 {f(v)} else {
        let newv = (1..(v.len() - 2)).map(|i| f(v[i..(i+5)].to_vec())).collect();

        nest(f, n - 1, newv)
    }
}

fn test1() {timeit!({
    // // Rule Odd
    // let f: F = Box::new(|v| if vec![vec![1, 0, 0], vec![0, 1, 0], vec![0, 0, 1], vec![1, 1, 1]].contains(&v) {1} else {0});
    // let mut s = Solver::from_single(3, f);

    // // Rule 108
    // let f: F = Box::new(|v| if vec![vec![1, 1, 0], vec![1, 0, 1], vec![0, 1, 1]].contains(&v) {1} else {0});
    // let mut s = Solver::from_seperated_any(3, f);

    // // Rule 108
    // let f: F = Box::new(|v| if vec![vec![1, 1, 0], vec![1, 0, 1], vec![0, 1, 1], vec![0, 1, 0]].contains(&v) {1} else {0});
    // let mut s = Solver::from_debruijn_parametric(3, f);
    
    // // Rule 30
    // let f: F = Box::new(|v| if vec![vec![1, 0, 0], vec![0, 1, 1], vec![0, 1, 0], vec![0, 0, 1]].contains(&v) {1} else {0});
    // let mut s = Solver::from_debruijn_parametric(3, f);
    
    // Code 20
    let f: F = Box::new(|v: Vec<usize>| if {let o: usize = v.iter().sum(); o == 2 || o == 4} {1} else {0});
    let mut s = Solver::from_debruijn_parametric(5, f);


    #[cfg(feature = "render")]
    s.fa.render_named_wl("sll");


    #[allow(unused)]
    let mut seen: HashSet<crate::fa::FA<usize>> = HashSet::new();

    for i in 1..20 {if timeit!({
        
        eprintln!("{i}");

        let sfa1 = timeit!({s.step().to_usize_fa()}, "stepping");
        #[cfg(feature = "render-all")]
        {sfa1.render_timestamped_wl("sfa1");}

        let sfa1 = timeit!({sfa1.remove_after_five_nodes().to_usize_fa()}, "pruning excess nodes1");
        // sfa1.f = sfa1.q.clone();
        #[cfg(feature = "render-all")]
        {sfa1.render_timestamped_wl("sfa1.5");}

        let sfa2 = timeit!({sfa1.bs_usize_to_dfa().to_usize_fa()}, "dfa conversion");
        #[cfg(feature = "render-all")]
        {sfa2.render_timestamped_wl("sfa2");}

        let sfa3 = timeit!({sfa2.remove_after_five_nodes()}, "pruning excess nodes");
        #[cfg(feature = "render-all")]
        {sfa3.render_timestamped_wl("sfa3");}

        // let sfa4 = timeit!({sfa3.usize_full_simplify()}, "simplifications");
        // #[cfg(feature = "render-all")]
        // {sfa4.render_timestamped_wl("sfa4");}

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