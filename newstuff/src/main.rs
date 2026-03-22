use hashbrown::{HashMap, HashSet}; 
use rayon::prelude::*;

// pub mod smallbitset;
// use smallbitset::SmallBitSet;

use bit_set::BitSet as SmallBitSet;

const K: usize = 2;

#[derive(Clone, Debug)]
struct Net {
    r: Vec<[Vec<usize>; K]>,
}

impl Net {
    fn ca_step(self, (r, rtab): (usize, Vec<usize>)) -> Net {
        let mut o = Vec::new();

        for n in self.r.iter() {
            for i in 0..(K.pow(2 * r as u32)) {
                let mut temp = [const { Vec::new() }; K];

                for a in 0..K {
                    for s in n[a].iter().copied() {
                        temp[rtab[i * K + a]].push(K.pow(2 * r as u32) * s + ((i * K + a) % (K.pow(2 * r as u32))));
                    }
                }
                
                o.push(temp)
            }
        }

        Net { r: o }
    }

    fn all_net() -> Net {
        let mut o = Net { r: vec![[const { vec![] }; K]] };
        
        for i in 0..K {o.r[0][i].push(0); }

        o
    }


/// Reverse all edges. Does NOT add a super-start node.
    fn reverse_edges(&self) -> Net {
        eprintln!("Reversing Started");

        let n = self.r.len();
        let mut r: Vec<[Vec<usize>; K]> = vec![std::array::from_fn(|_| Vec::new()); n];
        for s in 0..n {
            for a in 0..K {
                for &t in &self.r[s][a] {
                    r[t][a].push(s);
                }
            }
        }

        eprintln!("    Reversing Complete");

        Net { r }
    }

    /// Determinize via subset construction.
    /// `start_states`: the set of NFA states that form the initial DFA state.
    /// `accept_states`: which NFA states are accepting.
    /// Returns (DFA as Net, set of accepting DFA state indices).
    /// The dead state (empty set) is excluded; transitions to it become empty.
    fn determinize(
        &self,
        start_states: &[usize],
        accept_states: &HashSet<usize>,
    ) -> (Net, HashSet<usize>) {
        let n = self.r.len();

        eprintln!("Starting Determinization");

        // Precompute transition images as bitsets
        let images: Vec<[SmallBitSet; K]> = (0..n)
            .map(|s| {
                std::array::from_fn(|a| {
                    let mut bs = SmallBitSet::new();
                    for &t in &self.r[s][a] {
                        bs.insert(t);
                    }
                    bs
                })
            })
            .collect();

        eprintln!("    Step 1 Complete");

        // Precompute accept mask for O(WORDS) acceptance check
        // instead of iterating all accept_states
        let accept_mask: SmallBitSet = {
            let mut s = SmallBitSet::new();
            for &a in accept_states {
                s.insert(a);
            }
            s
        };

        let step = |set: &SmallBitSet, a: usize| -> SmallBitSet {
            let mut result = SmallBitSet::new();
            for s in set.iter() {
                result.union_with(&images[s][a]);
            }
            result
        };

        eprintln!("    Step 2 Complete");

        let start: SmallBitSet = {
            let mut s = SmallBitSet::new();
            for &st in start_states {
                s.insert(st);
            }
            s
        };

        let empty = SmallBitSet::new();

        let mut state_map: HashMap<SmallBitSet, usize> = HashMap::new();
        state_map.insert(start.clone(), 0);

        let mut queue: Vec<SmallBitSet> = vec![start];
        let mut transitions: Vec<[Option<usize>; K]> = Vec::new();
        let mut dfa_accept: HashSet<usize> = HashSet::new();

        let mut head = 0;

        let mut count = 0;

        while head < queue.len() {
            let batch_end = queue.len();
            let batch = &queue[head..batch_end];

            if batch.is_empty() {
                break;
            }

            // eprintln!(
            //     "        batch {head}..{batch_end} ({}), total {}",
            //     batch.len(),
            //     queue.len()
            // );

            eprintln!("        iter {count}, {} / {}, {:.2}%", batch.len(), queue.len(), 100. - 100. * batch.len() as f64 / queue.len() as f64 );
            count += 1;

            // Parallel: compute all K successors for each state in batch
            let successors: Vec<[SmallBitSet; K]> = batch
                .par_iter()
                .map(|current| std::array::from_fn(|a| step(current, a)))
                .collect();

            // Sequential: assign indices, record transitions, check acceptance
            for (i, succs) in successors.into_iter().enumerate() {
                let state_idx = head + i;

                // Bitwise accept check — O(WORDS) instead of O(|accept_states|)
                if !queue[state_idx].is_disjoint(&accept_mask) {
                    dfa_accept.insert(state_idx);
                }

                let trans: [Option<usize>; K] = std::array::from_fn(|a| {
                    if succs[a] == empty {
                        None
                    } else {
                        let next_idx =
                            if let Some(&idx) = state_map.get(&succs[a]) {
                                idx
                            } else {
                                let idx = queue.len();
                                state_map.insert(succs[a].clone(), idx);
                                queue.push(succs[a].clone());
                                idx
                            };
                        Some(next_idx)
                    }
                });
                transitions.push(trans);
            }

            head = batch_end;
        }

        eprintln!("    Step 3 Complete");

        let r = transitions
            .into_iter()
            .map(|trans| {
                std::array::from_fn(|a| match trans[a] {
                    Some(idx) => vec![idx],
                    None => vec![],
                })
            })
            .collect();

        eprintln!("    Step 4 Complete");

        (Net { r }, dfa_accept)
    }

    /// Brzozowski minimization.
    /// Original NFA: start = all states, accept = all states.
    fn minimize_brzozowski(&self) -> Net {
        let n = self.r.len();
        let all_states: Vec<usize> = (0..n).collect();
        let all_accept: HashSet<usize> = (0..n).collect();

        // Step 1: Reverse edges
        eprintln!("Step 1: reverse ({} states)", n);
        let rev1 = self.reverse_edges();

        // Step 2: Determinize reversed NFA
        // start = all (old accept = all), accept = all (old start = all)
        eprintln!("Step 2: determinize");
        let (det1, det1_accept) = rev1.determinize(&all_states, &all_accept);
        eprintln!("  {} states, {} accepting", det1.r.len(), det1_accept.len());

        // Step 3: Reverse the intermediate DFA
        eprintln!("Step 3: reverse");
        let rev2 = det1.reverse_edges();

        // Step 4: Determinize again
        // DFA start was state 0 → that's now an accept state
        // DFA accepts → those are now start states
        let det1_accept_vec: Vec<usize> = det1_accept.iter().copied().collect();
        let det1_start_set: HashSet<usize> = [0].into_iter().collect();

        eprintln!("Step 4: determinize");
        let (det2, _det2_accept) = rev2.determinize(&det1_accept_vec, &det1_start_set);
        eprintln!("  {} states", det2.r.len());

        det2
    }


    /// Compute the set of NFA states reachable from `state_set` on symbol `a`.
    fn net_step(&self, state_set: &[usize], a: usize) -> Vec<usize> {
        let mut result: Vec<usize> = state_set
            .iter()
            .flat_map(|&i| self.r[i][a].iter().copied())
            .collect();
        result.sort_unstable();
        result.dedup();
        result
    }

    fn select(&self, keep: &[usize]) -> Net {
        // Build old index -> new index mapping
        let mut old_to_new: HashMap<usize, usize> = HashMap::with_capacity(keep.len());
        for (new_idx, &old_idx) in keep.iter().enumerate() {
            old_to_new.insert(old_idx, new_idx);
        }

        let r = keep
            .iter()
            .map(|&old_idx| {
                std::array::from_fn(|a| {
                    self.r[old_idx][a]
                        .iter()
                        .filter_map(|&target| old_to_new.get(&target).copied())
                        .collect()
                })
            })
            .collect();

        Net { r }
    }

    fn without_unreachable(&self) -> Net {
        let mut reachable: Vec<usize> = vec![0];

        let mut numreachable = reachable.len();
        let mut prevnumreachable = numreachable + 1;

        while prevnumreachable != numreachable {
            let new: Vec<usize> = (0..K).flat_map(|a| self.net_step(&reachable, a).into_iter()).collect();
            reachable.extend_from_slice(&new);
            reachable.sort();
            reachable.dedup();

            prevnumreachable = numreachable;
            numreachable = reachable.len();
        }

        self.select(&reachable)
    }

    fn to_mathematica(&self) -> String {
        let o: Vec<Vec<String>> = self.r.iter().map(|edges|
            (0..K).map(|a|
                edges[a].iter().map(|n| format!("{a}->{}", n+1))
                    .collect::<Vec<String>>().join(",")
                ).collect::<Vec<String>>()
            ).collect();
        
        if o.len() > 1_000_000 {
            return format!("{{}}");
        }

        format!("{o:?}").replace("[", "{").replace("]", "}").replace("\"", "").replace("{, ", "{").replace(", }", "}")
    }

    fn paths(self, len: usize, max: usize) -> Vec<Path> {
        let mut o = vec![Path::new(0)];

        for i in 0..len {
            o = o.into_iter().flat_map(|path| 
                path.push_multiple(&self.r[path.nodes[i]]).into_iter()
            ).collect();

            o.sort();
            o.dedup();

            if o.len() > max {
                o = o[..max].to_vec();
                o.reverse();
            }
        }

        o
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
struct Path {
    nodes: Vec<usize>,
    collected: Vec<usize>
}

impl Path {
    fn new(start: usize) -> Self {
        Path {nodes: vec![start], collected: vec![] }
    }

    fn push(&mut self, node: usize, char: usize) {
        self.nodes.push(node);
        self.collected.push(char);
    }

    fn push_multiple(&self, netnode: &[Vec<usize>; K]) -> Vec<Self> {
        (0..K).flat_map(|char|
            netnode[char].clone().into_iter().map(move |node| {
                let mut o = self.clone();
                o.push(node, char);
                o
            })
        ).collect()
    }
}

fn mf(s: String, lab: &str, i: usize) -> String {
    format!("Export[\"renders/{lab}{i}.png\", Labeled[NetVisualize[{s}], \"{lab}{i}\"]];")
}

fn bits(n: usize, l: usize) -> Vec<usize> {
    assert!(n >> l == 0);   
    (0..l).map(|i| (n >> i) & 1).collect()
}

fn run_rule126() {
    // let code20rulenumber = 1771476584;

    let rule = (2, bits(1771476584, 32));
    // let rule = (1, bits(126, 8));

    let mut o = Net::all_net();

    println!("{}", mf(o.to_mathematica(), "0AllNet", 0));

    for iter in 0..5 {
        o = o.ca_step(rule.clone());
        println!("{}", mf(o.to_mathematica(), "1stepped", iter));
        o = o.without_unreachable();
        println!("{}", mf(o.to_mathematica(), "2withoutunreachable", iter));
        o = o.minimize_brzozowski();
        println!("{}", mf(o.to_mathematica(), "3minimized", iter));
    }

    let paths = o.paths(64, 32);
    let string = format!("Paths: {:?}", paths.into_iter().map(|path| path.collected).collect::<Vec<Vec<usize>>>()).replace("[", "{").replace("]", "}");
    eprintln!("{string}");
}

fn main() {
    println!("#!/usr/bin/env wolframscript");

    println!("NetVisualize[net_] :=  EdgeTaggedGraph[Range[Length[net]], Join @@ Table[Table[DirectedEdge[nodei, edge[[2]], edge[[1]]], {{edge, net[[nodei]]}}], {{nodei, Length[net]}}], VertexLabels -> \"Name\", EdgeLabels -> \"EdgeTag\", ImageSize -> Large]");

    run_rule126();
}
