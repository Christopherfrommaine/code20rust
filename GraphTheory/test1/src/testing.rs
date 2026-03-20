#[allow(unused_imports)]
use log::{debug, info};

use crate::solver::Solver;
use crate::timeit;

type F = Box<dyn Fn(Vec<usize>) -> usize + Send + Sync>;

fn code_20() -> F {
    Box::new(|v| {let o = v.into_iter().sum::<usize>(); if o == 2 || o == 4 {1} else {0}})
}
fn rule_n(n: u8) -> F {
    Box::new(move |v| if n & (1 << (4 * v[0] + 2 * v[1] + v[2])) != 0 {1} else {0})
}

#[allow(dead_code)]
fn test1() {timeit!({
    let args: Vec<String> = std::env::args().into_iter().skip(1).map(|s: String| s.trim().to_string()).collect();

    let n: usize;
    let f: F;
    if args.len() == 1 {
        n = 3;
        let rule_num = args[0].parse::<u8>().expect("Could not parse rule number.");
        f = rule_n(rule_num);
        info!("running with rule number {}", rule_num)
    } else {
        n = 5;
        f = code_20();
    }

    let mut s = Solver::from(n, f);

    #[cfg(feature = "render")]
    s.fa.render_timestamped_wl("sfa0");

    let mut seen = rustc_hash::FxHashSet::default();

    for i in 1..1r {if timeit!({
        
        eprintln!("{i}");

        let sfa1 = timeit!({s.step().to_usize_fa()}, "stepping");
        #[cfg(feature = "render-all")]
        {sfa1.render_timestamped_wl("sfa1");}

        let sfa2 = timeit!({sfa1.easy_simplifications()}, "easy simplifications"); // cant do full because NFA
        #[cfg(feature = "render-all")]
        {sfa2.render_timestamped_wl("sfa2");}

        let sfa3 = timeit!({sfa2.bs_usize_to_dfa().to_usize_fa()}, "dfa conversion");
        #[cfg(feature = "render-all")]
        {sfa3.render_timestamped_wl("sfa3");}

        let sfa4 = timeit!({sfa3.full_simplify()}, "simplifications 2");
        #[cfg(feature = "render-all")]
        {sfa4.render_timestamped_wl("sfa4");}

        let sfa5 = timeit!({sfa4.canonical_label()}, "canonize");
        #[cfg(feature = "render-all")]
        {sfa5.render_timestamped_wl("sfa5");}

        s.fa = sfa5;
        #[cfg(feature = "render")]
        s.fa.render_named_wl(&format!("sl{i}"));

        let o: bool = !seen.insert(s.fa.canonical_label());
        if o {info!("finished!");}
        o
    }, "full step") {break;} }
    println!("{},", format!("{:?}", s.without_null_loop().all_words(0, 0, 100, 0)).replace("[", "{").replace("]", "}"));

}, "full program");}


#[allow(dead_code)]
fn test2() {timeit!({
    let mut s = Solver::from(5, Box::new(|_| 0));
    s.fa = s.fa.bs_usize_to_dfa().to_usize_fa();

    #[cfg(feature = "render")]
    s.fa.render_timestamped_wl("sfa0");

    let mut seen = rustc_hash::FxHashSet::default();

    for i in 1..200 {if timeit!({
        
        eprintln!("{i}");

        let sfa1 = timeit!({s.fa.usize_dfa_step_2().to_usize_fa()}, "stepping");
        #[cfg(feature = "render-all")]
        {sfa1.render_timestamped_wl("sfa1");}

        let sfa2 = timeit!({sfa1.easy_simplifications()}, "easy simplifications"); // cant do full because NFA
        #[cfg(feature = "render-all")]
        {sfa2.render_timestamped_wl("sfa2");}

        let sfa3 = timeit!({sfa2.bs_usize_to_dfa().to_usize_fa()}, "dfa conversion");
        #[cfg(feature = "render-all")]
        {sfa3.render_timestamped_wl("sfa3");}

        let sfa4 = timeit!({sfa3.full_simplify()}, "simplifications 2");
        #[cfg(feature = "render-all")]
        {sfa4.render_timestamped_wl("sfa4");}

        let sfa5 = timeit!({sfa4.canonical_label()}, "canonize");
        #[cfg(feature = "render-all")]
        {sfa5.render_timestamped_wl("sfa5");}

        s.fa = sfa5;
        #[cfg(feature = "render")]
        s.fa.render_named_wl(&format!("sl{i}"));

        let o: bool = !seen.insert(s.fa.canonical_label());
        if o {info!("finished!");}
        o
    }, "full step") {break;} }
    println!("{},", format!("{:?}", s.without_null_loop().all_words(0, 0, 100, 0)).replace("[", "{").replace("]", "}"));

}, "full program");}


pub fn main() {
    test1();
}