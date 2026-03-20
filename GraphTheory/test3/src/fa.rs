use std::{collections::{HashMap, VecDeque}, fmt::Debug, hash::Hash};

use rustc_hash::FxHashMap;

#[derive(Clone, PartialEq, Eq)]
pub struct FA<N: Clone + Debug + Eq + Hash> {
    /// Nodes
    pub q: Vec<N>,
    /// Alphabet size
    pub sigma: usize,  // instead of the full alphabet, now the alphabet is derived from (0..sigma)
    /// Edges of the form (n1, captured, n2)
    pub t: Vec<(N, usize, N)>,
    /// Starting state
    pub q0: N,
    /// Halting states
    pub f: Vec<N>,

    /// Map of a node to a map of a charachter to a list of the nodes which follow and capture that charachter.
    /// The charachter map is just a vector, because charachters are just usize.
    pub precomputed_successors: HashMap<N, Vec<Vec<N>>>,
}

#[macro_export]
macro_rules! timeit {
    ( $block:block, $name:expr ) => {
        {
            let start_time = std::time::Instant::now();
            let result = $block;
            let duration = start_time.elapsed();
            log::info!("{} took {:?}", $name, duration);
            result
        }
    };
}


impl<N: Clone + Debug + Eq + Hash> FA<N> {
    pub fn from(q: Vec<N>, sigma: usize, t: Vec<(N, usize, N)>, q0: N, f: Vec<N>) -> Self {
        
        timeit!({
            let qset: rustc_hash::FxHashSet<N> = q.clone().into_iter().collect();

            assert!(qset.contains(&q0));  // starting state must be a node
            assert!(f.iter().all(|n| qset.contains(n)));  // all halting states must be nodes
            assert!(t.iter().all(|(n1, x, n2)| qset.contains(n1) && qset.contains(n2) && *x < sigma));
            assert!(f.len() != 0);
        }, "FA::from() asserts");

        let mut precomputed_successors: HashMap<N, Vec<Vec<N>>>;
        precomputed_successors = q.iter().map(|n| (n.clone(), vec![Vec::new(); sigma])).collect();
        t.iter().for_each(|(n1, x, n2)| {
            let v = &mut precomputed_successors.get_mut(&n1).unwrap()[*x];
            if !v.contains(n2) {v.push(n2.clone())}
        });

        FA {q, sigma, t, q0, f, precomputed_successors}
    }

    /// Directly construct an FA from it's properties
    pub fn from_raw(q: Vec<N>, sigma: usize, t: Vec<(N, usize, N)>, q0: N, f: Vec<N>, precomputed_successors: HashMap<N, Vec<Vec<N>>>) -> Self {

        debug_assert!(q.contains(&q0));  // starting state must be a node
        debug_assert!(f.iter().all(|n| q.contains(n)));  // all halting states must be nodes
        debug_assert!(t.iter().all(|(n1, x, n2)| q.contains(n1) && q.contains(n2) && *x < sigma));
        debug_assert!(f.len() != 0);

        FA {q, sigma, t, q0, f, precomputed_successors}
    }

    /// Get all following nodes that capture s
    #[allow(dead_code)]
    pub fn edges_of_node_with_state(&self, n1: &N, s: usize) -> &Vec<N> {
        &self.precomputed_successors.get(n1).unwrap()[s]
    }
    /// Get all following nodes
    #[allow(dead_code)]
    pub fn edges_of_node(&self, n1: &N) -> Vec<N> {
        (0..self.sigma)
        .flat_map(|s|
            self.edges_of_node_with_state(n1, s).iter()
        ).cloned().collect()
    }
    /// Get all following nodes as well as the value captured
    pub fn edges_of_node_capturing(&self, n1: &N) -> Vec<(usize, N)> {
        self.precomputed_successors.get(n1)
        .unwrap().clone().into_iter()
        .enumerate()
        .flat_map(|(i, v)| 
            v.into_iter()
            .map(move |x| (i, x))
        ).collect()
    }

    /// Returns a list of all iters-length paths and what they capture following the initial node.
    pub fn edge_tree(&self, node: (Vec<usize>, Vec<N>), iters: usize) -> Vec<(Vec<usize>, Vec<N>)> {

        // Uses a stack of searchers.
        let mut searchers = VecDeque::from(vec![(node.0, node.1, iters - 1)]);
        let mut output = vec![];

        // While stack has an item,
        while let Some((xs, ns, i)) = searchers.pop_front() {

            // For each possible node following ns[-1] capturing x
            for (x, n2) in self.edges_of_node_capturing(&ns[ns.len() - 1]) {
                
                // Create new_ns and new_xs with full path and capture sequence
                let mut new_ns = ns.clone();
                new_ns.push(n2);
                let mut new_xs = xs.clone();
                new_xs.push(x);

                // If path length reached, add to output. Otherwise, add to stack.
                if i == 0 {
                    output.push((new_xs, new_ns));
                } else {
                    searchers.push_back((new_xs, new_ns, i - 1));
                }
            }
        }

        output
    }

    /// Returns a list of all iters-length paths and what they capture following the initial node.
    pub fn edge_tree_of_node(&self, node: N, iters: usize) -> Vec<(Vec<usize>, Vec<N>)> {
        let     xs = Vec::with_capacity(iters - 1);
        let mut ns = Vec::with_capacity(iters);
        ns.push(node);
        
        self.edge_tree((xs, ns), iters)
    }

    // Maps each node to an integer
    pub fn to_usize_fa(&self) -> FA<usize> {
        let map: HashMap<N, usize> = self.q.clone().into_iter().enumerate().map(|(i, n)| (n, i)).collect();

        let new_q = (0..self.q.len()).collect();
        let new_sigma = self.sigma;
        let new_t = self.t.iter().map(|(n1, x, n2)| (*map.get(n1).unwrap(), *x, *map.get(n2).unwrap())).collect();
        let new_q0 = *map.get(&self.q0).unwrap();
        let new_f = self.f.iter().map(|n| *map.get(n).unwrap()).collect();
        
        // Optimization: use map to build sucuessors independently
        let new_ps = self.precomputed_successors.iter().map(|(n1, nsx2)|
            (
                *map.get(n1).unwrap(),
                nsx2.iter().map(|ns2: &Vec<N>|
                    ns2.iter().map(|n2: &N|
                        *map.get(n2).unwrap()
                    ).collect()
                ).collect()
            )
        ).collect();

        FA::from_raw(
            new_q,
            new_sigma,
            new_t,
            new_q0,
            new_f,
            new_ps,
        )
    }
}

impl FA<usize> {
    pub fn remove_after_five_nodes(&self) -> Self {
        const MAX_LEN: u32 = 5;

        let mut distance_map: FxHashMap<usize, u32> = self.q.iter().map(|n| (*n, MAX_LEN + 1)).collect();

        *distance_map.get_mut(&self.q0).unwrap() = 0;

        let mut changed = true;
        while changed {
            changed = false;

            for n1 in self.q.iter().copied() {
                let dist = *distance_map.get(&n1).unwrap();
                for n2 in self.edges_of_node(&n1) {
                    let curr_dist = *distance_map.get(&n2).unwrap();

                    if dist + 1 < curr_dist {
                        distance_map.insert(n2, dist + 1);
                        changed = true;
                    }
                }
            }

            println!("{:?}", self.q.iter().map(|n| (*n, *distance_map.get(n).unwrap())).collect::<Vec<(usize, u32)>>());
        }

        FA::from(
            self.q.iter().copied().filter(|n| *distance_map.get(n).unwrap() <= MAX_LEN).collect(),
            self.sigma,
            self.t.iter().copied().filter(|(n1, _x, n2)| *distance_map.get(n1).unwrap() <= MAX_LEN && *distance_map.get(n2).unwrap() <= MAX_LEN).collect(),
            self.q0,
            self.f.iter().copied().filter(|n| *distance_map.get(n).unwrap() <= MAX_LEN).collect(),
        )

    }
}