use crate::sat_solver::*;
use crate::{CNF, SATSolution};

extern crate time;
use time::{Instant, Duration};

/// A wrapper for another solver which exposes time performance
/// measurement
pub struct TimedSolver<S: Solver> {
    solver: S,
}

impl<S: Solver> Solver for TimedSolver<S> {
    fn solve(&self, formula: &CNF) -> SATSolution {
        self.solver.solve(formula)
    }
}

impl<S: Solver> TimedSolver<S> {
    /// Wraps a boxed solver in a `TimedSolver`
    pub fn new(solver: S) -> Self {
        TimedSolver{solver}
    }

    /// Runs the solver and returns the duration of the computation along with
    /// the actual result
    pub fn solve_timed(
        &self, formula: &CNF) -> (Duration, SATSolution) {
        let start = Instant::now();
        let solution = self.solve(formula);
        let end = Instant::now();
        (end - start, solution)
    }
}
