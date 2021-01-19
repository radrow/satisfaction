use crate::{CNF, SATSolution};
use rayon::prelude::*;


pub trait Solver {
    fn solve(&self, formula: CNF, num_variables: usize) -> SATSolution;
}

pub fn check_valuation(formula: &CNF, val: &Vec<bool>) -> bool {
    formula.clauses.par_iter()
        .all(|clause| clause.vars.iter().any(|var| var.sign() == val[var.id() - 1]))
}

impl<T: Solver> Solver for &T {
    fn solve(&self, formula: CNF, num_variables: usize) -> SATSolution {
        (*self).solve(formula, num_variables)
    }
}

impl<T: Solver> Solver for Box<T> {
    fn solve(&self, formula: CNF, num_variables: usize) -> SATSolution {
        (**self).solve(formula, num_variables)
    }
}
