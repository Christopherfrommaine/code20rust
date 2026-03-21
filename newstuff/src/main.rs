use hashbrown::{HashMap}; 
use rayon::prelude::*;

use dashmap::DashSet;

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

    fn dsets(self) -> Vec<Vec<usize>> {
        let n = self.r.len();

        let seen: DashSet<SmallBitSet> = DashSet::new();
        seen.insert(SmallBitSet::new());

        // let mut curr: Vec<SmallBitSet> = vec![SmallBitSet::from_range(n)];
        let mut curr: Vec<SmallBitSet> = vec![(0..n).collect()];

        let mut count = 0;
        let mut prevlen = 0;

        while !curr.is_empty() {

            eprintln!("    Iter {count}, {}, {}, {:.2}% Inc", seen.len(), curr.len(), 100. * curr.len() as f64 / prevlen as f64);
            count += 1;
            prevlen = curr.len();

            for s in &curr {
                seen.insert(s.clone());
            }

            eprintln!("    Part 1 complete.");

            let new_seen: DashSet<SmallBitSet> = DashSet::new();

            curr.par_iter().for_each(|nodes| {
                for a in 0..K {
                    let mut next = SmallBitSet::new();
                    for node in nodes.iter() {
                        for &target in &self.r[node][a] {
                            next.insert(target);
                        }
                    }
                    if !seen.contains(&next) && !new_seen.contains(&next) {
                        new_seen.insert(next);
                    }
                }
            });

            eprintln!("    Part 2 complete.");


            curr = new_seen.into_iter().collect();

            eprintln!("    Part 3 complete.");
        }

        let mut o: Vec<Vec<usize>> = seen.into_iter().map(|s| s.iter().collect()).collect();
        o.sort();
        o
    }

    fn isets(list: &[Vec<usize>]) -> Vec<Vec<usize>> {
        let n = list.len();
        if n == 0 {
            return vec![];
        }
        if n == 1 {
            return vec![vec![0]];
        }

        // Initial partition: {{0}, {1, 2, ..., n-1}}  (0-indexed)
        let mut g: Vec<Vec<usize>> = vec![vec![0], (1..n).collect()];

        loop {
            // Build mapping: index -> group number
            let mut group_of = vec![0usize; n];
            for (gi, group) in g.iter().enumerate() {
                for &idx in group {
                    group_of[idx] = gi;
                }
            }

            // Compute signature for each row:
            //   replace each element with the group number it belongs to
            let signatures: Vec<Vec<usize>> = list
                .iter()
                .map(|row| row.iter().map(|&elem| group_of[elem]).collect())
                .collect();

            // Refine: split each group by signature
            let mut new_g: Vec<Vec<usize>> = Vec::new();

            for group in &g {
                // Pair each index in this group with its signature
                let mut pairs: Vec<(&Vec<usize>, usize)> = group
                    .iter()
                    .map(|&idx| (&signatures[idx], idx))
                    .collect();

                // Sort by signature
                pairs.sort_by(|a, b| a.0.cmp(b.0));

                // Split into runs of equal signatures
                let mut i = 0;
                while i < pairs.len() {
                    let mut j = i + 1;
                    while j < pairs.len() && pairs[j].0 == pairs[i].0 {
                        j += 1;
                    }
                    let subgroup: Vec<usize> = pairs[i..j].iter().map(|&(_, idx)| idx).collect();
                    new_g.push(subgroup);
                    i = j;
                }
            }

            // Fixed point check
            if new_g == g {
                return g;
            }
            g = new_g;
        }
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

    /// Minimize the NFA by subset construction + partition refinement.
    ///
    /// Returns `None` if the dead state is unreachable (i.e., the automaton
    /// accepts everything — the "AllNet" case).
    ///
    /// Returns `Some(minimized_net)` otherwise, where transitions to the
    /// dead state are represented as empty `Vec`s.
    fn min_net(&self) -> Option<Net> {
        eprintln!("Minimizing:");

        let n = self.r.len();
        if n == 0 {
            return None;
        }

        // Step 1: Subset construction — compute reachable DFA states.
        // dsets() returns sorted subsets; d[0] == [] (the dead state)
        // if it is reachable.
        let d = self.clone().dsets();

        if d.is_empty() || !d[0].is_empty() {
            // Dead state not reachable => accepts all inputs => "AllNet"
            return None;
        }

        eprintln!("    Step 1 Complete");

        // Step 2: Build fast lookup from subset -> index in d.
        let d_index: HashMap<&Vec<usize>, usize> = d
            .iter()
            .enumerate()
            .map(|(i, s)| (s, i))
            .collect();

        eprintln!("    Step 2 Complete");

        // Step 3: Build DFA transition table.
        // b[i][a] = index in d of the state reached from d[i] on symbol a.
        let b: Vec<Vec<usize>> = d
            .par_iter()
            .map(|state_set| {
                (0..K)
                    .map(|a| {
                        let next = self.net_step(state_set, a);
                        *d_index
                            .get(&next)
                            .expect("NetStep produced a state not in DSets")
                    })
                    .collect()
            })
            .collect();

        eprintln!("    Step 3 Complete");

        // Step 4: Partition refinement on the DFA transition table.
        let q = Net::isets(&b);
        
        eprintln!("    Step 4.isets Complete");

        // Build reverse map: DFA state index -> equivalence class index.
        let mut class_of = vec![0usize; d.len()];
        for (ci, class) in q.iter().enumerate() {
            for &state in class {
                class_of[state] = ci;
            }
        }

        eprintln!("    Step 4 Complete");

        // Step 5: Build the minimized automaton.

        // The dead state is d[0] = []. Its equivalence class is the one
        // containing index 0.
        let dead_class = class_of[0];

        // Assign new contiguous indices to live (non-dead) equivalence classes.
        let mut new_index = vec![0usize; q.len()];
        let mut count = 0usize;
        for ci in 0..q.len() {
            if ci != dead_class {
                new_index[ci] = count;
                count += 1;
            }
        }

        eprintln!("    Step 5.1 Complete");

        // Build the minimized transition table.
        let mut r: Vec<[Vec<usize>; K]> = Vec::with_capacity(count);
        for ci in 0..q.len() {
            if ci == dead_class {
                continue;
            }
            let rep = q[ci][0]; // representative of this equivalence class
            let transitions: [Vec<usize>; K] = std::array::from_fn(|a| {
                let target_class = class_of[b[rep][a]];
                if target_class != dead_class {
                    vec![new_index[target_class]]
                } else {
                    vec![] // transition to dead state — omitted
                }
            });
            r.push(transitions);
        }

        eprintln!("    Step 5.2 Complete");

        // Identify the start state: the equivalence class of the initial
        // DFA state. The initial DFA state is the full set {0, 1, ..., n-1},
        // which is the last element of d (since d is sorted and this is the
        // largest subset that was seeded).
        let initial_dfa_state: Vec<usize> = (0..n).collect();
        let initial_index = *d_index
            .get(&initial_dfa_state)
            .expect("Initial state set not found in DSets");
        let start_class = class_of[initial_index];
        let start = new_index[start_class];

        eprintln!("    Step 5.3 Complete");

        // If start is already 0, we're done. Otherwise, swap row 0 and
        // row `start`, and update all references.
        if start != 0 {
            r.swap(0, start);
            // Fix up all transition targets: anything pointing to 0
            // should now point to `start` and vice versa.
            for row in r.iter_mut() {
                for a in 0..K {
                    for target in row[a].iter_mut() {
                        if *target == 0 {
                            *target = start;
                        } else if *target == start {
                            *target = 0;
                        }
                    }
                }
            }
        }

        eprintln!("    Step 5.4 Complete");

        Some(Net { r })
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

    // let rule = (2, bits(1771476584, 32));
    let rule = (1, bits(126, 8));

    let mut o = Net::all_net();

    println!("{}", mf(o.to_mathematica(), "0AllNet", 0));

    for iter in 0..4 {
        o = o.ca_step(rule.clone());
        println!("{}", mf(o.to_mathematica(), "1stepped", iter));
        o = o.without_unreachable();
        println!("{}", mf(o.to_mathematica(), "2withoutunreachable", iter));
        o = o.min_net().expect("whoops");
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
