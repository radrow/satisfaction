use crate::{cnf, dpll};
use auto_impl::auto_impl;
use cnf::{VarId, CNFVar};
use dpll::{Clauses, Clause, VarValue, Variables, Variable};
use std::cmp::Ordering;
use std::collections::HashMap;

type ClauseId = usize;

#[auto_impl(&mut, Box)]
pub trait CDCLDeletionStrategy {
    /// Function that returns clause indices to be deleted
    fn gc_clauses(&mut self, clauses: &Clauses) -> Vec<ClauseId>;

    fn learn_clause(&mut self, clause: Clause);

    fn conflict_contrib_hook(&mut self, index: ClauseId);
}


struct BerkMin {
    activity: HashMap<ClauseId, u32>,
    threshold: u32,
}

impl BerkMin {
    pub fn new() -> BerkMin {
        BerkMin {
            activity: HashMap::new(),
            threshold: 60,
        }
    }
}

impl CDCLDeletionStrategy for BerkMin {
    fn gc_clauses(&mut self, clauses: &Clauses) -> Vec<ClauseId> {
        let young = &clauses[0..&clauses.len() / 16];
        let old = &clauses[&clauses.len() / 16..clauses.len()];

        let (young_out, young_in) : (Vec<(usize, &Clause)>, Vec<(usize, &Clause)>) = young
            .iter()
            .enumerate()
            .partition(|(i, c)| c.active_lits > 42 && self.activity[i] < 7);
        let (old_out, old_in) : (Vec<(usize, &Clause)>, Vec<(usize, &Clause)>) = old
            .iter()
            .enumerate()
            .partition(|(i, c)| c.active_lits > 8 && self.activity[i] < self.threshold);

        self.threshold += 1;

        self.activity = old_in.iter().chain(young_in.iter())
            .enumerate()
            .map(|(new_i, (i, _))| (new_i, self.activity[i]))
            .collect();

        old_out.iter().chain(young_out.iter()).map(|(i, _)| *i).collect()
    }

    fn learn_clause(&mut self, _clause: Clause) {
        self.activity.entry(self.activity.len()).or_insert(0);
    }

    fn conflict_contrib_hook(&mut self, index: ClauseId) {
        self.activity.entry(index).and_modify(|x| *x += 1).or_insert(1);
    }
}
