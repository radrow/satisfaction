use crate::sat_solver::*;
use crate::{SATSolution, Valuation, CNF};

/// A simple CNF solver that naively checks all possible
/// valuations in order to ensure satisfiability
pub enum Bruteforce {
    Bruteforce,
}

impl Solver for Bruteforce {
    fn solve(&self, formula: &CNF) -> SATSolution {
        // initial valuation sets all to false
        let mut valuation = Vec::new();
        for _ in 0..formula.num_variables {
            valuation.push(false);
        }
        if guess(&formula, 0, &mut valuation) {
            SATSolution::Satisfiable(valuation)
        } else {
            SATSolution::Unsatisfiable
        }
    }
}

fn guess(formula: &CNF, change: usize, valuation: &mut Valuation) -> bool {
    if change == valuation.len() {
        check_valuation(formula, valuation)
    } else {
        if guess(formula, change + 1, valuation) {
            true
        } else {
            // set current bit
            valuation[change] = true;
            // try again
            let res = guess(formula, change + 1, valuation);
            if !res {
                // if failed set back to default
                valuation[change] = false;
            }
            res
        }
    }
}
