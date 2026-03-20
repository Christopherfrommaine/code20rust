use std::{collections::HashMap, fmt::Debug, hash::Hash};
use rustc_hash::FxHashMap;

use crate::bitset::BitSet;

pub struct GFA<N: Clone + Debug + Eq + Hash> {
    pub q: Vec<N>,
    pub t: Box<dyn Fn(N, N) -> u8>,
    pub q0: N,
    pub f: Vec<N>,
    pub precomputed_successors: HashMap<N, Vec<Vec<N>>>,
}

impl<N: Clone + Debug + Eq + Hash> GFA<N> {
    pub fn to_usize_fa(&self) -> FA {
        let map: FxHashMap<N, usize> = self.q.clone().into_iter().enumerate().map(|(i, n)| (n, i)).collect();

        let new_q0 = *map.get(&self.q0).unwrap();
        let new_f = self.f.iter().map(|n| *map.get(n).unwrap()).collect();

        FA::from(
            (0..self.q.len()).collect(),
            (move |a, b| self.t(*map.get(a).unwrap(), *map.get(b).unwrap()),
            new_q0,
            new_f,
        )
    }
}

pub struct FA {
    pub q: Vec<usize>,
    pub t: Box<dyn Fn(usize, usize) -> u8>,
    pub q0: usize,
    pub f: Vec<usize>,
    pub precomputed_successors: HashMap<usize, (BitSet, BitSet)>,
}

#[macro_export]
macro_rules! timeit {
    ( $block:block, $name:expr ) => {
        {
            let start_time = std::time::Instant::now();
            let result = $block;
            let duration = start_time.elapsed();
            log::debug!("{} took {:?}", $name, duration);
            result
        }
    };
}


impl FA {
    pub fn from(q: Vec<usize>, t: , q0: N, f: Vec<N>) -> Self {
        
        let qset: rustc_hash::FxHashSet<N> = q.clone().into_iter().collect();
        assert!(qset.contains(&q0));  // starting state must be a node
        assert!(f.iter().all(|n| qset.contains(n)));  // all halting states must be nodes
        assert!(t.iter().all(|(n1, x, n2)| qset.contains(n1) && qset.contains(n2) && *x < sigma));
        assert!(f.len() != 0);

        let mut precomputed_successors: HashMap<N, Vec<Vec<N>>>;
        precomputed_successors = q.iter().map(|n| (n.clone(), vec![Vec::new(); sigma])).collect();
        t.iter().for_each(|(n1, x, n2)| {
            let v = &mut precomputed_successors.get_mut(&n1).unwrap()[*x];
            if !v.contains(n2) {v.push(n2.clone())}
        });

        FA {q, sigma, t, q0, f, precomputed_successors}
    }

    pub fn from_raw(q: Vec<N>, sigma: usize, t: Vec<(N, usize, N)>, q0: N, f: Vec<N>, precomputed_successors: HashMap<N, Vec<Vec<N>>>) -> Self {

        debug_assert!(q.contains(&q0));  // starting state must be a node
        debug_assert!(f.iter().all(|n| q.contains(n)));  // all halting states must be nodes
        debug_assert!(t.iter().all(|(n1, x, n2)| q.contains(n1) && q.contains(n2) && *x < sigma));
        debug_assert!(f.len() != 0);

        FA {q, sigma, t, q0, f, precomputed_successors}
    }

    // Get all nodes with a specific state
    #[allow(dead_code)]
    pub fn edges_of_node_with_state(&self, n1: &N, s: usize) -> &Vec<N> {
        &self.precomputed_successors.get(n1).unwrap()[s]
    }
    // Get all following nodes
    #[allow(dead_code)]
    pub fn edges_of_node(&self, n1: &N) -> Vec<N> {
        (0..self.sigma)
        .flat_map(|s|
            self.edges_of_node_with_state(n1, s).iter()
        ).cloned().collect()
    }
    // Get all following nodes and the edges they capture
    pub fn edges_of_node_capturing(&self, n1: &N) -> Vec<(usize, N)> {
        self.precomputed_successors.get(n1)
        .unwrap().clone().into_iter()
        .enumerate()
        .flat_map(|(i, v)| 
            v.into_iter()
            .map(move |x| (i, x))
        ).collect()
    }

    pub fn edge_tree(&self, edge: (Vec<usize>, Vec<N>), iters: usize) -> Vec<(Vec<usize>, Vec<N>)> {
        let mut searchers = vec![(edge.0, edge.1, iters - 1)];
        let mut output = vec![];

        while let Some((xs, ns, i)) = searchers.pop() {
            for (x, n2) in self.edges_of_node_capturing(&ns[ns.len() - 1]) {
                let mut new_ns = ns.clone();
                new_ns.push(n2);
                let mut new_xs = xs.clone();
                new_xs.push(x);

                if i == 0 {
                    output.push((new_xs, new_ns));
                } else {
                    searchers.push((new_xs, new_ns, i - 1));
                }
            }
        }

        output
    }

    pub fn edge_tree_bin(&self, edge: (u8, Vec<N>), iters: usize) -> Vec<(u8, Vec<N>)> {
        let mut searchers = vec![(edge.0, edge.1, iters - 1)];
        let mut output = vec![];

        while let Some((xs, ns, i)) = searchers.pop() {
            for (x, n2) in self.edges_of_node_capturing(&ns[ns.len() - 1]) {
                let mut new_ns = ns.clone();
                new_ns.push(n2);
                let new_xs = xs << 1 | x as u8;

                if i == 0 {
                    output.push((new_xs, new_ns));
                } else {
                    searchers.push((new_xs, new_ns, i - 1));
                }
            }
        }

        output
    }

    
}