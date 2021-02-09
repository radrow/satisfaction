use std::time::Duration;
use async_std::future::timeout;
use async_std::task::block_on;
use async_trait::async_trait;
use super::InterruptibleSolver;
use crate::{SATSolution, CNF, Solver};


pub struct TimeLimitedSolver<S: InterruptibleSolver> {
    max_duration: Duration,
    solver: S,
}

impl<S: InterruptibleSolver> TimeLimitedSolver<S> {
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
