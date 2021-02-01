use std::time::Duration;
use futures::executor::block_on;
use tokio::time::timeout;
use async_trait::async_trait;
use super::InterruptibleSolver;
use crate::{SATSolution, CNF, Solver};


/// A wrapper for an `InterruptibleSolver`
/// that cancels execution after a specified period of time.
pub struct TimeLimitedSolver<S: InterruptibleSolver> {
    max_duration: Duration,
    solver: S,
}

impl<S: InterruptibleSolver> TimeLimitedSolver<S> {
    /// Convenient function for creating  `TimeLimitedSolver`.
    ///
    /// # Arguments
    /// * `solver` - An `InterruptibleSolver` which is wrapped.
    /// * `max_duration` - Maximal time how long the wrapped solver is allowed to run.
    ///
    pub fn new(solver: S, max_duration: Duration) -> TimeLimitedSolver<S> {
        TimeLimitedSolver {
            solver,
            max_duration,
        }
    }
}

impl<S: InterruptibleSolver> Solver for TimeLimitedSolver<S> {
    fn solve(&self, formula: &CNF) -> SATSolution {
        block_on(async {
            timeout(self.max_duration, self.solver.solve_interruptible(formula)).await
                .unwrap_or(SATSolution::Unknown)
        })
    }
}

#[async_trait]
impl <S: InterruptibleSolver+Send+Sync> InterruptibleSolver for TimeLimitedSolver<S> {
    async fn solve_interruptible(&self, formula: &CNF) -> SATSolution {
        timeout(self.max_duration, self.solver.solve_interruptible(formula)).await
            .unwrap_or(SATSolution::Unknown)
    }
}
