use crate::fa::FA;
use crate::cellularautomation::bits;
use log::{debug, trace};

pub struct Solver {
    pub fa: FA<usize>,
    neighbors: usize,
    rule: Box<dyn Fn(Vec<usize>) -> usize + Send + Sync>,
}

impl Solver {
    /// Generates a solver from the FA capturing everything
    #[allow(dead_code)]
    pub fn from_any(n: usize, rule: Box<dyn Fn(Vec<usize>) -> usize + Send + Sync>) -> Solver {

        let original = FA::from(vec![0], 2, vec![(0, 0, 0), (0, 1, 0)], 0, vec![0]);

        Solver {
            fa: original,
            neighbors: n,
            rule
        }
    }

    /// Generates a solver from the FA capturing everything, with two nodes instead of 1
    #[allow(dead_code)]
    pub fn from_debruijn_parametric(neighbors: usize, rule: Box<dyn Fn(Vec<usize>) -> usize + Send + Sync>) -> Solver {

        let q: Vec<usize> = (0..(2usize.pow(neighbors as u32 - 1))).collect();
        let mut t = Vec::new();
        for n1 in q.iter().copied() {
            for n2 in q.iter().copied() {
                let mask = 2usize.pow(neighbors as u32 - 2) - 1;
                trace!("{}, {}, {}, {:?}, {}", mask & n1, n2 >> 1, (n1 << 1) | n2, bits((n1 << 1) | n2, neighbors), rule(bits((n1 << 1) | n2, neighbors)));
                if (mask & n1) == n2 >> 1 {
                    t.push((n1, rule(bits((n1 << 1) | n2, neighbors)), n2));
                }
            }
        }
        debug!("q: {q:?}");
        debug!("t: {t:?}");

        let original = FA::from(
            q, 
            2, 
            t,
            0, 
            vec![0]
        );

        #[cfg(feature = "render-all")]
        {original.render_named_wl("debruijn original");}

        let minimized = original.full_simplify();

        #[cfg(feature = "render-all")]
        {minimized.render_named_wl("debruijn minimized");}

        Solver {
            fa: minimized,
            neighbors,
            rule
        }
    }

    /// Generates a solver from the input 00000100000 (etc)
    #[allow(dead_code)]
    pub fn from_single(n: usize, rule: Box<dyn Fn(Vec<usize>) -> usize + Send + Sync>) -> Solver {

        let original = FA::from(vec![0, 1], 2, vec![(0, 0, 0), (0, 1, 1), (1, 0, 1)], 0, vec![1]);

        Solver {
            fa: original,
            neighbors: n,
            rule
        }
    }

    /// Generates a solver from a debruijn starting graph
    #[allow(dead_code)]
    pub fn from_debruijn(n: usize, rule: Box<dyn Fn(Vec<usize>) -> usize + Send + Sync>) -> Solver {

        let original = FA::debruign(3, &rule);

        Solver {
            fa: original,
            neighbors: n,
            rule
        }
    }
 

    pub fn step(&mut self) -> FA<(Vec<usize>, Vec<usize>)> {
        self.fa.step(self.neighbors, &self.rule)
    }

    /// Redirects all edges towards q[0] to a new node
    #[allow(dead_code)]
    pub fn without_null_loop(&self) -> FA<usize> {
        let fa = &self.fa;

        let next = fa.q.len();

        let mut new_q = fa.q.clone();
        new_q.push(next);
        let new_t: Vec<(usize, usize, usize)> = fa.t.clone().into_iter().map(|(n1, x, n2)| {
            if n2 == fa.q0 {
                (n1, x, next)
            } else {
                (n1, x, n2)
            }
        }).collect();
        let mut new_f = fa.f.clone();
        new_f.push(next);
        let new_q0 = fa.q0;

        FA::from(
            new_q,
            fa.sigma,
            new_t,
            new_q0,
            new_f,
        )
    }
}