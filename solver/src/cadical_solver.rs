use cadical;
use crate::{Solver, SATSolution, CNF};

pub struct CadicalSolver;

impl Solver for CadicalSolver {
    fn solve(&self, formula: &CNF) -> SATSolution {
        let mut solver: cadical::Solver = Default::default();

        let num_variables = formula.num_variables;
        for clause in &formula.clauses {
            let mut lits = Vec::new();
            lits.reserve(clause.vars.len());
            for var in &clause.vars {
                lits.push(var.to_i32())
            }
            solver.add_clause(lits.into_iter());
        }

        match solver.solve() {
            None => SATSolution::Unknown,
            Some(false) => SATSolution::Unsatisfiable,
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
