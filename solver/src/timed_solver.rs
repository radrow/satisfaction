use crate::sat_solver::*;
use crate::{CNF, SATSolution};
use std::time::{Instant, Duration};
use crate::TimeLimitedSolver;

/// A wrapper for another solver which exposes time performance
/// measurement
pub struct TimedSolver<S> {
    solver: S,
}

impl<S: Solver> Solver for TimedSolver<S> {
    fn solve(&self, formula: &CNF) -> SATSolution {
        self.solver.solve(formula)
    }
}

impl<S> TimedSolver<S> {
    /// Wraps a boxed solver in a `TimedSolver`
    pub fn new(solver: S) -> Self {
        TimedSolver{solver}
    }
}


impl<S: TimeLimitedSolver> TimedSolver<S> {
    pub fn solve_timed(&self, formula: &CNF, max_duration: Duration) -> (Duration, SATSolution) {
        let start = Instant::now();
        let solution = self.solver.solve(formula, max_duration);
        let duration = start.elapsed();
        (duration, solution)
    }
}
