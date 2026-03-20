use crate::fa::FA;

pub struct Solver {
    pub fa: FA<usize>,
    pub n: usize,
    rule: Box<dyn Fn(u8) -> u8 + Send + Sync>,
}

impl Solver {
    pub fn from(n: usize, rule: Box<dyn Fn(Vec<usize>) -> usize + Send + Sync>) -> Solver {
        let original = FA::debruign(n, &rule);

        Solver {
            fa: original,
            n,
            rule
        }
    }

    pub fn step(&mut self) -> FA<(Vec<usize>, Vec<usize>)> {
        self.fa.step(self.n, &self.rule)
    }

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
            new_t,
            new_q0,
            new_f,
        )
    }
}