use crate::{SATSolution, CNF};
use auto_impl::auto_impl;
use rayon::prelude::*;

/// Abstraction over a SAT-Solver:
/// Each solver is expected to receive a `CNF` `formula` with all variable IDs being greater than 0
/// and output a SATSolution.
/// If the formula was satisfied it returns a Vec of booleans representing
/// a contiguous range from variable with ID 1 (index 0) to the variable with the maximal ID).
#[auto_impl(Box)]
pub trait Solver {
    fn solve(&self, formula: &CNF) -> SATSolution;
}

pub fn check_valuation(formula: &CNF, val: &Vec<bool>) -> bool {
    formula.clauses.par_iter().all(|clause| {
        clause
            .vars
            .iter()
            .any(|var| var.sign() == val[var.id() - 1])
    })
}
