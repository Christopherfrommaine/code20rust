use log::{trace, info};
use rayon::prelude::*;

use crate::fa::FA;
use std::{collections::VecDeque, fmt::Debug, hash::Hash};
use rustc_hash::{FxHashSet, FxHashMap};


impl<N: Clone + Debug + Hash + Eq + Ord + Send + Sync> FA<N> {
    /// Performs a DFA Conversion
    pub fn to_dfa(&self) -> FA<Vec<N>> {
        // DFA Conversion:
        // Go from FA<N> to FA<Vec<N>>
        // Starts with [q0]
        // This connects to two nodes: a vec of the followers of all q0 capturing 0,
        // and a vec of the followers of q0 capturing 1.
        // Each new node (a vec of old nodes) connects to the node
        // of the vec of all the x-followers of all the old nodes

        type NewNode<N> = Vec<N>;

        info!("q_size: {}", self.q.len());
        info!("t_size: {}", self.t.len());

        let new_q0 = vec![self.q0.clone()];
        let mut new_q: FxHashSet<NewNode<N>> = FxHashSet::default();
        new_q.insert(new_q0.clone());
        let mut new_t: Vec<(NewNode<N>, usize, NewNode<N>)> = Vec::new();

        // List of new nodes which have not been explored yet
        let mut searchers: Vec<NewNode<N>> = vec![new_q0.clone()];
        while let Some(s) = searchers.pop() {
            // #[cfg(debug_assertions)]
            

            for x in 0..self.sigma {
                // Given a start new_node and the char to capture,

                // All nodes that might follow:
                let mut following: Vec<N> = s.par_iter().flat_map_iter(|n1| self.edges_of_node_with_state(n1, x).into_iter().cloned()).collect();

                if following.len() != 0 {
                    // Searchers can grow to the millions
                    // println!("searchers: {}, f: {}", searchers.len(), following.len());

                    // Ensure uniqueness
                    following.sort_unstable();
                    following.dedup();

                    if !new_q.contains(&following) {
                        // Add new_node to graph
                        new_q.insert(following.clone());
                        searchers.push(following.clone());
                    }
                    new_t.push((s.clone(), x, following));
                }
                
            }
        }

        let new_f: Vec<NewNode<N>> = new_q.iter().filter(|vn| vn.iter().any(|n| self.f.contains(n))).cloned().collect();
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

impl FA<usize> {
    /// Optimized dfa conversion for FA<usize>
    #[allow(dead_code)]
    pub fn usize_to_dfa(&self) -> FA<Vec<usize>> {
        info!("q_size: {}", self.q.len());

        let new_q0 = vec![self.q0];
        let mut new_q: FxHashSet<Vec<usize>> = FxHashSet::default();
        new_q.insert(vec![self.q0]);
        let mut new_t = Vec::new();

        let mut searchers = vec![vec![self.q0]];
        while let Some(s) = searchers.pop() {

            for x in 0..self.sigma {
                let mut following: Vec<usize> = s.iter().flat_map(|n1| self.edges_of_node_with_state(n1, x).into_iter().copied()).collect();

                if following.len() != 0 {
                    following.sort_unstable();
                    following.dedup();

                    if !new_q.contains(&following) {
                        new_q.insert(following.clone());
                        searchers.push(following.clone());
                    }
                    
                    new_t.push((s.clone(), x, following));
                }
                
            }
        }

        let new_f = new_q.iter().filter(|vn| vn.iter().any(|n| self.f.contains(n))).cloned().collect();
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

use crate::bitset::BitSet;
impl FA<usize> {
    /// Super optimized dfa conversion for FA<usize>
    #[allow(dead_code)]
    pub fn bs_usize_to_dfa(&self) -> FA<BitSet> {
        info!("q_size: {}", self.q.len());
        info!("t_size: {}", self.t.len());

        if self.q.len() >= 100 {
            assert!(false, "too bad so sad you're all out of RAM");
        }

        let new_q0 = BitSet::from_vec(&[self.q0], self.q.len());
        let mut new_q: FxHashSet<BitSet> = FxHashSet::default();
        new_q.insert(new_q0.clone());
        let mut new_t = Vec::new();

        let mut searchers = vec![new_q0.clone()];

        let transitions: Vec<Vec<BitSet>> = (0..self.q.len()).map(|n| {
            (0..self.sigma).map(|x|
                BitSet::from_vec(self.edges_of_node_with_state(&n, x), self.q.len())
            ).collect()
        }).collect();

        let mut following = BitSet::with_capacity(self.q.len());
        while let Some(s) = searchers.pop() {

            // if searchers.len() % 10_000 == 0 {
            //     println!("{}", searchers.len());
            // }

            for x in 0..self.sigma {

                following.clear();

                for n1 in s.iter() {
                    following.or_inplace(&transitions[n1][x]); // wordwise OR of u64 words
                }

                if !following.is_empty() {
                    if new_q.insert(following.clone()) {
                        // If following is newly inserted
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
        let mut prev = FA::from(vec![self.q0.clone()], 2, vec![], self.q0.clone(), vec![self.q0.clone()]);
        let mut curr = self.clone();

        while prev != curr {
            let new_curr = curr.remove_unproductive_nodes().remove_unreachable_nodes();
            prev = curr;
            curr = new_curr;
            info!("easy_simp_loop...")
        }

        curr
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
        info!("Starting DFA Minimization");
        
        // For faster membership checks probably
        let fset: FxHashSet<N> = self.f.clone().into_iter().collect();

        // Map of original states to reduced states. Starts as identity map.
        let mut map: FxHashMap<&N, &N> = self.q.iter().map(|n| (n, n)).collect();

            // seen outgoing edges
            // The tuple (is_halting_state, outgoing_edges) uniquely identifies the behavior of a node
            // By grouping together nodes with the same behavior, 
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
    pub fn usize_full_simplify(&self) -> Self {
        self.easy_simplifications().usize_dfa_minimize()
    }

    pub fn usize_dfa_minimize(&self) -> Self {
        // Node type. For this, I'm only implementing it where node labels are of type usize.
        type N = usize;

        let mut new_q = self.q.clone();
        let mut new_t = self.t.clone();
        let mut new_f = self.f.clone();
        let mut new_q0 = self.q0.clone();
        let mut outgoing_edges: FxHashMap<N, Vec<(usize, N)>> = new_q.iter().map(|n| (*n, self.edges_of_node_capturing(n))).collect();

        let mut prev_q_size = 0;
        let mut q_size = new_q.len();

        while prev_q_size != q_size {

            // For faster lookups
            let fset: FxHashSet<N> = new_f.iter().cloned().collect();

            // The behavior of a state is uniquely identified by (is_halting, outgoing_edges)
            let mut behavior_set: FxHashSet<(bool, Vec<(usize, N)>)> = FxHashSet::default();

            let behavior_of = |n: N| {
                let is_halting = fset.contains(&n);
                let outgoing_edges: Vec<(usize, N)> = outgoing_edges.get(&n).unwrap().clone();
                let behavior: (bool, Vec<(usize, N)>) = (is_halting, outgoing_edges);

                behavior
            };

            for n in new_q.iter().cloned() {
                behavior_set.insert(behavior_of(n));
            }

            // Maps a behavior to an index
            let behavior_map: FxHashMap<(bool, Vec<(usize, N)>), usize> = behavior_set.into_iter().enumerate().map(|(i, v)| (v, i)).collect();
            let old_to_new = |n: N| behavior_map.get(&behavior_of(n)).unwrap().clone();

            new_q = (0..behavior_map.len()).collect();
            new_t = new_t.into_iter().map(|(n1, x, n2)| {
                (old_to_new(n1), x, old_to_new(n2))
            }).collect::<FxHashSet<(N, usize, N)>>().into_iter().collect();
            new_f = new_f.into_iter().map(old_to_new).collect::<FxHashSet<usize>>().into_iter().collect();
            new_q0 = old_to_new(new_q0);
            outgoing_edges = outgoing_edges.clone().into_iter().map(|(k, v)|
                (
                    old_to_new(k),
                    {let mut o: Vec<(usize, N)> = v.into_iter().map(|(x, following)| (x, old_to_new(following))).collect(); o.sort_unstable(); o.dedup(); o}
                )
            ).collect();


            (prev_q_size, q_size) = (q_size, new_q.len());
        }

        FA::from(new_q, self.sigma, new_t, new_q0, new_f)
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

        eprint!("");
        self.render_timestamped_wl("what??");
        println!("{:?}", self.q);
        println!("{:?}", self.t);
        

        let qmap: FxHashMap<usize, usize> = o.into_iter().zip(self.q.iter().cloned()).map(|(i, n)| (n, i)).collect();
        println!("qmap: {:?}", qmap);
        println!("{:?}", qmap.keys());
        debug_assert_eq!(FxHashSet::from_iter(self.q.iter().cloned()), FxHashSet::from_iter(qmap.keys().cloned().into_iter()));
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