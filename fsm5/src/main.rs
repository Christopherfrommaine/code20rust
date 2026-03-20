mod known_automata;

use known_automata::known_automata;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn castep(inp: &Vec<u8>) -> Vec<u8> {
    let thing: &mut Vec<u8> = &mut vec![0, 0];
    thing.extend_from_slice(inp);
    thing.push(0);
    thing.push(0);

    (0..inp.len()).map(|i| {
        let sum = thing[i] + thing[i + 1] + thing[i + 2] + thing[i + 3] + thing[i + 4];
        if sum == 2 || sum == 4 {1} else {0}
    }).collect()
}

fn carun(inp: Vec<u8>, t: usize) -> Vec<Vec<u8>> {
    let mut o = vec![inp];
    for _ in 0..t {
        o.push(castep(&o[o.len() - 1]));
    }
    o
}

fn largest_sequential_zeros(list: Vec<u8>) -> usize {
    list.split(|&x| x != 0)
        .map(|group| group.len())
        .max()
        .unwrap_or(0)
}

fn sequence_count<T: PartialEq>(list: &[T], sub: &[T]) -> usize {
    if sub.is_empty() || sub.len() > list.len() {
        return 0;
    }
    list.windows(sub.len()).filter(|w| *w == sub).count()
}

fn remove_surrounding_zeros(v: Vec<u8>) -> Vec<u8> {
    let start = v.iter().position(|&x| x != 0).unwrap_or(0);
    let end = v.iter().rposition(|&x| x != 0).map_or(0, |i| i + 1);
    if start >= end {
        Vec::new()
    } else {
        v[start..end].to_vec()
    }
}

fn method8(p: usize) -> Vec<Vec<u8>> {

    fn m8r(s: Vec<u8>, p: usize, working: &mut Vec<Vec<u8>>) {
        let slen = s.len();
        let centerbitpos = slen - 2 * p - 1;

        let mut spad = vec![0; 2 * p];
        spad.extend_from_slice(&s);
        spad.extend_from_slice(&vec![0; 2 * p]);


        let fullca: Vec<Vec<u8>> = carun(spad, p);

        let centerinvariant = if centerbitpos < 4 * p + 1 {0} else {if centerbitpos == 4 * p + 1 {1} else {s[centerbitpos]}} == fullca[p][centerbitpos + 2 * p];
        let condensed: Vec<u8> = (0..(slen + 4 * p)).map(|ci| (0..p).map(|ri| fullca[ri][ci]).fold(0, |a, b| a | b)).collect();
        
        let unseperated = largest_sequential_zeros(condensed[(6 * p + 1)..(slen + 4 * p - 2 * p - 1)].to_vec()) < 2 * p + 1;
        let notknown = known_automata().into_iter().map(|bv| sequence_count(&s, &bv)).fold(0, |a, b| a + b) < 1;
        let nonzero = true; // we'll see

        let total = centerinvariant && unseperated && notknown && nonzero;
        
        if total {
            if remove_surrounding_zeros(s.clone()) == remove_surrounding_zeros(fullca[p].clone()) {
                working.push(s.clone());
            } else {
                let mut s1 = s.clone();
                let mut s2 = s.clone();
                s1.push(0); s2.push(1);
                m8r(s1, p, working);
                m8r(s2, p, working);
            }
        }

    }

    let mut s0 = vec![0; 4 * p + 1];
    s0.push(1);

    let mut working = Vec::new();
    m8r(s0, p, &mut working);

    working

}


fn normal(sv: Vec<Vec<u8>>, p: usize) -> Vec<Vec<u8>> {
    let mut seen = Vec::new();
    let mut o = Vec::new();

    for s in sv {
        let mut thing = vec![0; 2 * p + 1];
        thing.extend_from_slice(&s);
        thing.extend_from_slice(&vec![0; 2 * p + 1]);

        let fullca = carun(thing, p);

        if fullca[1..p].iter().all(|r| remove_surrounding_zeros(r.to_vec()) != remove_surrounding_zeros(fullca[0].to_vec())) {
            if !seen.contains(&remove_surrounding_zeros(s.clone())) {
                o.push(s);

                seen.extend(fullca.into_iter().map(|r| remove_surrounding_zeros(r)));
            }
        }
    }

    o
}

fn plotallnormalca(inp: Vec<Vec<u8>>, p: usize) {
    normal(inp, p).into_iter().for_each(|s| {
        let mut thing = vec![0; 2 * p + 1];
        thing.extend_from_slice(&s);
        thing.extend_from_slice(&vec![0; 2 * p + 1]);

        let fullca = carun(thing, 5 * p);

        let title = format!("\n--- {p} ---");

        let mut plot = cgrustplot::plots::array_plot::array_plot(&fullca);
        plot.set_axes(false);
        plot.set_title(&title);

        plot.print();

        let path = format!("{}plots/p{p}{:?}.png", cgrustplot::helper::file::get_current_dir(), remove_surrounding_zeros(s)).replace("[", "a").replace("]", "b").replace(",", "c").replace(" ", "d");
        plot.as_image().save(&path);

    });
}

fn method8t(p: usize) -> Vec<Vec<u8>> {

    use std::thread;
    use std::sync::mpsc;
    
    let (tx, rx) = mpsc::channel();

    // stack size = 4GB
    let stack_size = 4_000_000_000;

    let handle = thread::Builder::new()
        .stack_size(stack_size)
        .spawn(move || {
            let result = method8(p);
            tx.send(result).expect("Failed to send result");
        })
        .expect("Failed to spawn thread");

    handle.join().expect("Thread panicked");

    let result = rx.recv().expect("Failed to receive result");

    result
}


fn all(p: usize) {
    let sol = method8t(p);
    // println!("{sol:?}");
    println!("p: {p} | {:?}", normal(sol.clone(), p));
    plotallnormalca(sol, p);
    // sol.into_iter().for_each(|s| {
    //     plotca(s, p);
    // });
}

fn main() {
    (1..100).into_par_iter().for_each(|p| all(p));
}