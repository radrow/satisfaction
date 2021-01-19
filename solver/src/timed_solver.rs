use crate::sat_solver::*;
use crate::{CNF, SATSolution};

extern crate time;
use time::{Instant, Duration};

/// A wrapper for another solver which exposes time performance
/// measurement
pub struct TimedSolver {
    solver: Box<dyn Solver>
}

impl Solver for TimedSolver {
    fn solve(&self, formula: CNF, num_variables: usize) -> SATSolution {
        self.solver.solve(formula, num_variables)
    }
}

impl TimedSolver {
    /// Wraps a boxed solver in a `TimedSolver`
    pub fn new(solver: Box<dyn Solver>) -> TimedSolver {
        TimedSolver{solver: solver}
    }

    /// Runs the solver and returns the duration of the computation along with
    /// the actual result
    pub fn solve_timed(
        &self, formula: CNF, num_variables: usize) -> (Duration, SATSolution) {
        let start = Instant::now();
        let solution = self.solve(formula, num_variables);
        let end = Instant::now();
        (end - start, solution)
    }
}
