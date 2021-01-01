use cadical;
use crate::{CNFClause, Solver, Assignment};

pub struct CadicalSolver;

impl Solver for CadicalSolver {
    fn solve(&self, clauses: impl Iterator<Item=CNFClause>, num_variables: usize) -> Option<Assignment> {
        let mut solver: cadical::Solver = Default::default();
        
        clauses.for_each(|clause| {
            solver.add_clause(clause.into_iter()
                .map(|literal| literal.to_i32()));
        });

        match solver.solve() {
            None | Some(false) => None,
            Some(true) => {
                // TODO: Use more index independent formulation
                Some((1..num_variables)
                    .map(|variable| {
                        solver.value(variable as i32)
                            // If None, the variable can be choosen arbitrarily and thus true. TODO: Discuss behaviour.
                            .unwrap_or(true) 
                    }).collect())
            }
        }
    }
}