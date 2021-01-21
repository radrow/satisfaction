use std::time::Duration;

use crate::{CNF, SATSolution};

pub trait TimeLimitedSolver {
    fn solve(&self, formula: &CNF, max_duration: Duration) -> SATSolution;
}

impl TimeLimitedSolver for Box<dyn TimeLimitedSolver> {
    fn solve(&self, formula: &CNF, max_duration: Duration) -> SATSolution {
        self.as_ref().solve(formula, max_duration)
    }
}

impl<S: TimeLimitedSolver> TimeLimitedSolver for Box<S> {
    fn solve(&self, formula: &CNF, max_duration: Duration) -> SATSolution {
        self.as_ref().solve(formula, max_duration)
    }
}
