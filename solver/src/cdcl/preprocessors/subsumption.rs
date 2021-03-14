use crate::{CNF, CNFVar, CNFClause, SATSolution};
use super::Preprocessor;


fn sig(clause: CNFClause, num_vars: usize) -> usize {
    let mut mask = 1;
    let mut result = 0;
    for i in 0..num_vars {
        if clause.vars.contains(&CNFVar {id: i, sign: true}) || clause.vars.contains(&CNFVar {id: i, sign: false}) {
            result = result | mask;
        }
        mask = mask << 1;
    }
    result
}

fn subsumption_test(clause1: CNFClause, clause2: CNFClause, num_vars: usize) -> bool {
    if sig(clause1, num_vars) & !sig(clause2, num_vars) != 0 {
        return false;
    }
    true
}
