use cadical;
use crate::{Solver, Assignment, CNF};

pub struct CadicalSolver;

impl Solver for CadicalSolver {
    fn solve(&self, clauses: CNF) -> Assignment {
        let mut solver: cadical::Solver = Default::default();
        
        let num_variables = clauses.num_variables;
        clauses.clauses.into_iter().for_each(|clause| {
            solver.add_clause(clause.into_iter()
                .map(|literal| literal.to_i32()));
        });

        match solver.solve() {
            None => Assignment::Unknown,
            Some(false) => Assignment::Unsatisfiable,
            Some(true) => {
                // TODO: Use more index independent formulation
                (1..=num_variables)
                    .map(|variable| {
                        solver.value(variable as i32)
                            // If None, the variable can be choosen arbitrarily and thus true. TODO: Discuss behaviour.
                            .unwrap_or(false) 
                    }).collect()
            }
        }
    }
}
