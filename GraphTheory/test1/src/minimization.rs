#[allow(unused_imports)]
use log::{debug, info};

use crate::fa::FA;
use std::{collections::VecDeque, fmt::Debug, hash::Hash};
use rustc_hash::{FxHashSet, FxHashMap};

use crate::bitset::BitSet;
impl FA<usize> {
    pub fn bs_usize_to_dfa(&self) -> FA<BitSet> {
        info!("dfa conversion q_size: {}", self.q.len());

        let new_q0 = BitSet::from_vec(&[self.q0], self.q.len());
        let mut new_q: FxHashSet<BitSet> = FxHashSet::default();
        new_q.insert(new_q0.clone());
        let mut new_t = Vec::new();

        let mut searchers = vec![new_q0.clone()];

        let mut following = BitSet::with_capacity(self.q.len());
        while let Some(s) = searchers.pop() {

            for x in 0..self.sigma {

                following.clear();
                
                for n1 in s.iter() {
                    following.extend(self.edges_of_node_with_state(&n1, x));
                }

                if !following.is_empty() {

                    if !new_q.contains(&following) {
                        new_q.insert(following.clone());
                        searchers.push(following.clone());
                    }
                    
                    new_t.push((s.clone(), x, following.clone()));
                }
                
            }
        }

        let new_f = new_q.iter().filter(|vn| vn.iter().any(|n| self.f.contains(&n))).cloned().collect();
        let new_sigma = self.sigma;

        FA::from(
            new_q.into_iter().collect(),
            new_sigma,
            new_t,
            new_q0,
            new_f,
        )

    }
}


impl<N: Clone + Debug + Hash + Eq> FA<N> {
    pub fn easy_simplifications(&self) -> Self {
        self
        .remove_unproductive_nodes()
        .remove_unreachable_nodes()
        .remove_unproductive_nodes()
        .remove_unreachable_nodes()
        .remove_unproductive_nodes()
    }

    pub fn full_simplify(&self) -> Self {
        self.easy_simplifications().dfa_minimize()
    }

    pub fn remove_unproductive_nodes(&self) -> Self {
        let mut productive: FxHashSet<N> = self.f.clone().into_iter().collect();
        let mut prev_len = 0;

        while productive.len() > prev_len {
            prev_len = productive.len();

            for n1 in self.q.iter() {
                if self.edges_of_node(n1).into_iter().any(|n2| productive.contains(&n2)) {
                    productive.insert(n1.clone());
                }
            }
        }

        let new_t = self.t.iter().filter(|(n1, _x, n2)| productive.contains(n1) && productive.contains(n2)).cloned().collect();
        let new_q = productive.into_iter().chain([self.q0.clone()].into_iter()).collect();
        let new_sigma = self.sigma;
        let new_q0 = self.q0.clone();
        let new_f = self.f.clone();

        FA::from(
            new_q,
            new_sigma,
            new_t,
            new_q0,
            new_f,
        )
    }

    pub fn remove_unreachable_nodes(&self) -> Self {
        let mut reachable = FxHashSet::default();
        reachable.insert(self.q0.clone());
        let mut searchers = vec![self.q0.clone()];

        while let Some(n1) = searchers.pop() {
            self.edges_of_node(&n1).into_iter().for_each(|n2| {
                if !reachable.contains(&n2) {
                    if !searchers.contains(&n2) {
                        searchers.push(n2.clone());
                    }
                    reachable.insert(n2);
                }
            });
        }

        let new_sigma = self.sigma;
        let new_q0 = self.q0.clone();
        let new_t = self.t.iter().filter(|(n1, _x, n2)| reachable.contains(n1) && reachable.contains(n2)).cloned().collect();
        let new_f = self.f.iter().filter(|n| reachable.contains(n)).cloned().collect();
        let new_q = reachable.into_iter().collect();

        FA::from(
            new_q,
            new_sigma,
            new_t,
            new_q0,
            new_f,
        )
    }

    pub fn dfa_minimize(&self) -> Self {

        let fset: FxHashSet<N> = self.f.clone().into_iter().collect();

        // Map of original states to reduced states. Starts as identity map
        let mut map: FxHashMap<&N, &N> = self.q.iter().map(|n| (n, n)).collect();

        // seen outgoing edges
        let mut seen_outgoing: FxHashMap<(bool, Vec<(usize, N)>), &N> = self.q.iter().map(|n| ((fset.contains(n), self.edges_of_node_capturing(n)), n)).collect();

        let mut changed = true;
        while changed {
            changed = false;

            for n1  in map.iter().map(|(_k, v)| v).cloned().collect::<Vec<&N>>() {
                let outgoing = (fset.contains(n1), self.edges_of_node_capturing(n1));

                // If there was an existing value
                // i.e. if this node has equivalent outgoing edges to some already seen node
                if let Some(v) = seen_outgoing.get(&outgoing) {
                    if n1 != v.to_owned() {
                        // map the original state to an equivalent state
                        *map.get_mut(n1).unwrap() = v;
                        changed = true;
                    }
                }
                
            }

            seen_outgoing = seen_outgoing.into_iter().map(|((c, v), n1)|
                ((c, v.into_iter().map(|(x, n)|
                    (x, map.get(&n).unwrap().to_owned().to_owned())
                ).collect()), n1)).collect();
        }

        let new_q0 = map.get(&self.q0).unwrap().to_owned().to_owned();
        let new_f = self.f.iter().map(|n| map.get(n).unwrap().to_owned().to_owned()).collect();
        let new_q = map.into_iter().map(|(_k, v)| v).cloned().collect::<FxHashSet<N>>().into_iter().collect();
        let new_t = seen_outgoing.clone().into_iter().flat_map(|(e, n1)| e.1.into_iter().map(|(x, n2)| (n1.clone(), x, n2))).collect();
        let new_sigma = self.sigma;

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
    pub fn canonical_label(&self) -> Self {
        type N = usize;

        let mut o: Vec<N> = vec![self.q0];
        let mut seen: FxHashSet<N> = FxHashSet::default();
        let mut searchers: VecDeque<N> = VecDeque::from([self.q0]);
        seen.insert(self.q0);

        while let Some(n1) = searchers.pop_front() {
            for x in 0..self.sigma {

                let n2s = self.edges_of_node_with_state(&n1, x);
                assert!(n2s.len() <= 1);  // DFA, not an NFA

                for n2 in n2s {
                    if seen.insert(*n2) {
                        o.push(*n2);
                        searchers.push_back(*n2);
                    }
                }
            }
        }

        let qmap: FxHashMap<usize, usize> = o.into_iter().enumerate().map(|(i, n)| (n, i)).collect();
        let new_q0 = 0;
        let new_q = (0..self.q.len()).collect();
        let mut new_t: Vec<(usize, usize, usize)> = self.t.iter().map(|(n1, x, n2)| (*qmap.get(n1).unwrap(), *x, *qmap.get(n2).unwrap())).collect();
        new_t.sort_unstable_by_key(|(n1, x, n2)| self.q.len() * self.sigma * n1 + self.q.len() * x + n2 );  // sorted by n1 is most significant, then x1, then n2. Relatively arbitrary but n1 closer together might make more cache hits during precomputation?
        let new_sigma = self.sigma;
        let mut new_f: Vec<usize> = self.f.iter().map(|n| *qmap.get(n).unwrap()).collect();
        new_f.sort_unstable();

        FA::from(
            new_q,
            new_sigma,
            new_t,
            new_q0,
            new_f,
        )
    }
}