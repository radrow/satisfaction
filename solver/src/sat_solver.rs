use crate::{CNF, SATSolution};
use rayon::prelude::*;
use auto_impl::auto_impl;


#[auto_impl(Box)]
pub trait Solver {
    fn solve(&self, formula: &CNF) -> SATSolution;
}

pub fn check_valuation(formula: &CNF, val: &Vec<bool>) -> bool {
    formula.clauses.par_iter()
        .all(|clause| clause.vars.iter().any(|var| var.sign() == val[var.id() - 1]))
}
