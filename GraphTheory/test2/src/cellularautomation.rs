use std::collections::{HashMap, HashSet};
use std::{fmt::Debug, hash::Hash};
use crate::fa::*;
use rayon::prelude::*;
use log::info;
use crate::timeit;

fn bits(inp: usize, length: usize) -> Vec<usize> {
    (0..length as u32).map(|l| inp.rotate_right(l) & 1).rev().collect()
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

        // stupid that it requires a generic argument
        type EdgePath<N> = (Vec<usize>, Vec<N>);
        
        // new_q = n-hyperedges of the connections of self
        let new_q: Vec<EdgePath<N>> = timeit!({self.q.clone().into_par_iter().flat_map_iter(|start| {
            self.edge_tree((vec![], vec![start]), neighbors - 1).into_iter()

        }).collect()}, "step new_q");

        println!("n_expected: {}, n_real: {}, n_used: {}", neighbors, new_q[0].1.len(), new_q[0].1.capacity());
        
        let mut qmap: rustc_hash::FxHashMap<Vec<usize>, Vec<usize>> = (0..(self.sigma.pow(neighbors as u32 - 1))).map(|i| (bits(i, neighbors - 1), Vec::new())).collect();

        new_q.iter().enumerate().for_each(|(i, (xs, _ns))| {
            qmap.get_mut(xs).unwrap().push(i);
        });

        // todo: memory intensive
        // O(new_q.len() * sigma * neighbors * path_length)
        let mut new_t: Vec<(EdgePath<N>, usize, EdgePath<N>)> = timeit!({new_q.iter().flat_map(|n1| {
            (0..self.sigma)
            .flat_map(|x| {
                let mut full_xs = n1.0.clone();
                full_xs.push(x);

                let full_xs_o = rule(full_xs.clone());

                qmap.get(&full_xs[1..]).unwrap()
                .into_iter()
                .cloned()
                .map(|n2i| new_q[n2i].clone())
                .map(|n2| {
                    debug_assert!(n1.0[1..] == n2.0[..(neighbors - 2)]);
    
                    (n1.clone(), full_xs_o, n2.clone())
                }).collect::<Vec<(EdgePath<N>, usize, EdgePath<N>)>>().into_iter()
            })
        }).collect()}, "step new_t");

        let new_sigma = 2;
        let new_q0: EdgePath<N> = new_q[0].clone();  // should be ([0, 0], [...])
        let new_f: Vec<EdgePath<N>> = vec![new_q0.clone()];

        FA::from(
            new_q,
            new_sigma,
            new_t,
            new_q0,
            new_f,
        )
    }

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


use smallvec::SmallVec;
impl FA<usize> {
    pub fn usize_step(&self, neighbors: usize, rule: &Box<dyn Fn(u8) -> usize + Send + Sync>) -> FA<(u8, SmallVec<[usize; 8]>)> {
        assert!(self.sigma == 2);

        type EdgePath = (u8, SmallVec<[usize; 8]>);

        // assumes max neighbors length of <=7 for storage in a u8
        assert!(neighbors <= 7);
        
        // new_q = n-hyperedges of the connections of self
        let new_q: Vec<EdgePath> = timeit!({self.q.clone().into_par_iter().flat_map_iter(|start| {
            self.edge_tree_bin((0, vec![start]), neighbors - 1).into_iter().map(|(xs, ns)| (xs, SmallVec::from_vec(ns)))
        }).collect()}, "step new_q");
        
        // the all-new hashless hash map!
        let qmask = (1 << (neighbors - 1)) - 1;
        let mut qmap: Vec<Vec<usize>> = vec![Vec::new(); self.sigma.pow(neighbors as u32 - 1)];

        new_q.iter().enumerate().for_each(|(i, (xs, _ns))| {
            qmap[*xs as usize].push(i);
        });


        info!("new_q.len(): {}", new_q.len());
        info!("qmap_avg_len: {}", qmap.iter().map(|v| v.len()).sum::<usize>() as f64 / qmap.len() as f64);


        let mut new_t: Vec<(EdgePath, usize, EdgePath)> = Vec::new();
        timeit!({for n1 in new_q.iter() {
            let xs: u8 = n1.0 << 1;
            let xs_o: usize = rule(xs);

            for n2i in qmap[(qmask & xs) as usize].iter()  {
                new_t.push((n1.clone(), xs_o, new_q[*n2i].clone()));
            };

            let xs: u8 = (n1.0 << 1) + 1;
            let xs_o: usize = rule(xs);

            for n2i in qmap[(qmask & xs) as usize].iter()  {
                new_t.push((n1.clone(), xs_o, new_q[*n2i].clone()));
            };
        }}, "step new_t");

        let new_sigma = 2;
        let new_q0: EdgePath = new_q[0].clone();  // should be ([0, 0], [...])
        let new_f: Vec<EdgePath> = vec![new_q0.clone()];

        FA::from(
            new_q,
            new_sigma,
            new_t,
            new_q0,
            new_f,
        )
    }
}


impl FA<usize> {
    pub fn usize_dfa_step(&self, neighbors: usize, rule: &Box<dyn Fn(u8) -> usize + Send + Sync>) -> FA<u8> {
        assert!(self.sigma == 2);

        type EdgePath = u8;

        // assumes max neighbors length of <=7 for storage in a u8
        assert!(neighbors <= 7);
        
        // new_q = n-hyperedges of the connections of self
        let new_q: Vec<EdgePath> = timeit!({self.q.clone().into_par_iter().flat_map_iter(|start| {
            self.edge_tree_bin((0, vec![start]), neighbors - 1).into_iter().map(|(xs, ns)| xs) 
        }).collect()}, "step new_q");
        
        // the all-new hashless hash map!
        let qmask = (1 << (neighbors - 1)) - 1;
        let mut qmap: Vec<Vec<usize>> = vec![Vec::new(); self.sigma.pow(neighbors as u32 - 1)];

        new_q.iter().enumerate().for_each(|(i, xs)| {
            qmap[*xs as usize].push(i);
        });

        info!("new_q.len(): {}", new_q.len());
        info!("qmap_avg_len: {}", qmap.iter().map(|v| v.len()).sum::<usize>() as f64 / qmap.len() as f64);


        let mut new_t: Vec<(EdgePath, usize, EdgePath)> = Vec::new();
        timeit!({for n1 in new_q.iter() {
            let xs: u8 = n1 << 1;
            let xs_o: usize = rule(xs);

            for n2i in qmap[(qmask & xs) as usize].iter()  {
                new_t.push((n1.clone(), xs_o, new_q[*n2i].clone()));
            };

            let xs: u8 = (n1 << 1) + 1;
            let xs_o: usize = rule(xs);

            for n2i in qmap[(qmask & xs) as usize].iter()  {
                new_t.push((n1.clone(), xs_o, new_q[*n2i].clone()));
            };
        }}, "step new_t");

        let new_sigma = 2;
        let new_q0: EdgePath = new_q[0].clone();  // should be ([0, 0], [...])
        let new_f: Vec<EdgePath> = vec![new_q0.clone()];

        FA::from(
            new_q,
            new_sigma,
            new_t,
            new_q0,
            new_f,
        )
    }
}

