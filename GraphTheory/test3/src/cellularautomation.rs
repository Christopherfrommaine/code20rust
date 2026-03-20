use std::collections::HashSet;
use std::{fmt::Debug, hash::Hash};
use crate::fa::*;
use rayon::prelude::*;
use log::{info, debug};
use crate::timeit;

#[allow(dead_code)]
/// Bits of a number n. Basically IntegerDigits[inp, 2, length]
pub fn bits(inp: usize, length: usize) -> Vec<usize> {
    (0..length as u32).map(|l| inp.rotate_right(l) & 1).rev().collect()
}

#[allow(dead_code)]
pub fn code20step(v: &Vec<usize>) -> usize {
    let o: usize = v.iter().sum();
    if o == 2 || o == 4 {1} else {0}
}

impl FA<usize> {
    pub fn debruign(neighbors: usize, rule: &Box<dyn Fn(Vec<usize>) -> usize + Send + Sync>) -> FA<usize> {

        let mask = (1 << (neighbors - 1)) - 1;
        
        let new_q: Vec<usize> = (0..(1 << (neighbors - 1))).collect();
        let new_sigma = 2;
        let new_t = (0..(1 << (neighbors - 1))).flat_map(|i: usize|
            [0, 1].into_iter().map(move |x| {

                let mut bits: Vec<usize> = (0..(neighbors - 1)).map(|j| i.rotate_right(j as u32) & 1 ).rev().collect();
                bits.push(x & 1);

                let o = (
                    i,
                    rule(bits),
                    mask & ((i << 1) | x)
                );

                o
            })
        ).collect();
        let new_q0 = 0;
        let new_f = vec![0];

        FA::from(
            new_q,
            new_sigma,
            new_t,
            new_q0,
            new_f,
        )
    }
}

impl<N: Clone + Debug + Hash + Eq + Send + Sync> FA<N> {
    pub fn step(&self, neighbors: usize, rule: &Box<dyn Fn(Vec<usize>) -> usize + Send + Sync>) -> FA<(Vec<usize>, Vec<N>)> {

        assert!(self.q.len() != 0);

        // stupid that it requires a generic argument
        type EdgePath<N> = (Vec<usize>, Vec<N>);
        
        // new_q = n-hyperedges of the connections of self
        let new_q: Vec<EdgePath<N>> = timeit!({self.q.clone().into_par_iter().flat_map_iter(|start| {
            let et = self.edge_tree_of_node(start, neighbors - 1);

            et.into_iter()

        }).collect()}, "step new_q");
        debug!("new_q: {new_q:?}");

        let new_t: Vec<(EdgePath<N>, usize, EdgePath<N>)> = timeit!({
            new_q.iter().flat_map(|n1| {
                new_q.iter()
                .filter(|n2| n1.1[1..] == n2.1[..(neighbors - 1)])
                .map(|n2| {

                    // println!("n1: {:?}, n2: {:?}", n1.clone(), n2.clone());

                    let mut xs = n1.0.clone();
                    xs.push(n2.0[neighbors - 2]);

                    // println!("xs: {xs:?}");

                    let o = rule(xs);

                    (n1.clone(), o, n2.clone())
                })
            }).collect()
        }, "step new_t");

        debug!("new_t: {new_t:?}");

        let new_sigma = 2;
        let new_q0 = new_q[0].clone();
        // let new_q0_should_be = (vec![0; neighbors - 1], vec![self.q0.clone(); neighbors]);
        // assert_eq!(new_q0, new_q0_should_be);
        // let new_f: Vec<EdgePath<N>> = vec![new_q0.clone()];
        let new_f: Vec<EdgePath<N>> = new_q.iter()
            .filter(|(_xs, ns)| self.f.contains(&ns[neighbors - 1]))
            .cloned()
            .collect();

        FA::from(
            new_q,
            new_sigma,
            new_t,
            new_q0,
            new_f,
        )
    }

    /// Return an FA that captures only sequences which self AND other may capture.
    #[allow(dead_code)]
    pub fn language_intersection<M: Clone + Debug + Hash + Eq + Send + Sync>(&self, other: &FA<M>) -> FA<(N, M)> {
        
        let new_q: Vec<(N, M)> = self.q.iter().flat_map(|q1| other.q.iter().map(|q2| (q1.clone(), q2.clone()))).collect();

        info!("language intersection with {} nodes", new_q.len());

        let new_sigma = self.sigma.max(other.sigma);
        let new_q0 = (self.q0.clone(), other.q0.clone());
        let new_f: Vec<(N, M)> = self.f.iter().flat_map(|f1| other.f.iter().map(|f2| (f1.clone(), f2.clone()))).collect();
        let new_t: Vec<((N, M), usize, (N, M))> = self.q.iter().flat_map(|n1|
            other.q.iter().flat_map(|n2|
                (0..new_sigma).filter_map(|x| {
                    match (self.edges_of_node_with_state(n1, x).first(), other.edges_of_node_with_state(n2, x).first()) {
                        (Some(q1), Some(q2)) => Some(((n1.clone(), n2.clone()), x, (q1.clone(), q2.clone()))),
                        _ => None,
                    }
                })
            )
        ).collect();

        info!("and {} edges.", new_t.len());

        let o = FA::from(
            new_q,
            new_sigma,
            new_t,
            new_q0,
            new_f
        );

        debug!("language_intersection output: {:?}", o);

        o.render_timestamped_wl("test");

        o
    }

    pub fn all_words(&self, max_paths: usize, max_iters: usize, max_solutions: usize, max_loops: usize) -> Vec<Vec<usize>> {
        let mut searchers: HashSet<(Vec<N>, Vec<usize>)> = HashSet::new();
        searchers.insert((vec![self.q0.clone()], Vec::new()));
        let mut found: HashSet<Vec<usize>> = HashSet::new();

        let mut iter = 0;
        while searchers.len() != 0 && (searchers.len() < max_paths || max_paths == 0) && (iter < max_iters || max_iters == 0) && (found.len() < max_solutions || max_solutions == 0) {
            iter += 1;
            debug!("searchers: {:?}", searchers.len());

            let mut new_searchers = HashSet::new();
            for (ns, xs) in searchers {
                let n1 = ns.last().unwrap();

                for (x, n2) in self.edges_of_node_capturing(n1) {

                    let mut o = xs.clone();
                    o.push(x);
                    
                    if ns.clone().into_iter().map(|nsi| if nsi == n2 {1} else {0}).fold(0, |a, b| a + b) > max_loops {
                        found.insert(o);
                        continue;
                    }

                    if self.f.contains(&n2) {
                        found.insert(o.clone());
                    }
                    
                    let mut new_ns = ns.clone();
                    new_ns.push(n2);

                    new_searchers.insert((new_ns, o));
                }
            }
            
            searchers = new_searchers;
        }

        found.into_iter().collect()
    }
}