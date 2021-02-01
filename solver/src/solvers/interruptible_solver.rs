use crate::{CNF, SATSolution, Solver};
use async_trait::async_trait;
use async_std::task::block_on;
use auto_impl::auto_impl;

#[async_trait]
#[auto_impl(Box)]
pub trait InterruptibleSolver {
    async fn solve_interruptible(&self, formula: &CNF) -> SATSolution;
}

pub struct InterruptibleSolverWrapper<S: InterruptibleSolver> {
    solver: S,
}

impl<S: InterruptibleSolver> From<S> for InterruptibleSolverWrapper<S> {
    fn from(solver: S) -> Self {
        InterruptibleSolverWrapper{solver}
    }
}

impl<S: InterruptibleSolver> Solver for InterruptibleSolverWrapper<S> {
    fn solve(&self, formula: &CNF) -> SATSolution {
        block_on(self.solver.solve_interruptible(formula))
    }
}
