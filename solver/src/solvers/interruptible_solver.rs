use crate::{SATSolution, Solver, CNF};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread::{JoinHandle, spawn};
use std::future::Future;
use std::task::{Poll, Context, Waker};
use std::pin::Pin;
use async_std::task::block_on;
use async_trait::async_trait;
use auto_impl::auto_impl;

/// A solver that can be interrupted at fixed location
/// determined by its instances.
/// More precisely, `solve_interruptible` is expected to transfer execution control
/// to its caller reasonable often.
/// This is useful for e.g. `TimeLimitedSolver` and aborting a solving process to achieve a
/// responsive GUI.
/// Because of its nature it is not guaranteed that
/// it is no guaranteed to stop immediately.
///
/// See `SatisfactionSolver`for an example.
#[async_trait]
#[auto_impl(Box, &)]
pub trait InterruptibleSolver: Send + Sync {
    /// A method for solving CNF-formulae following the same specifcations as `Solver::solve`.
    /// However, this function can transfer execution control to its caller.
    async fn solve_interruptible(&self, formula: &CNF) -> SATSolution;
}

/// A Wrapper that allows every `InterruptibleSolver` to derive a `Solver`-Implementation.
/// This is a convenience as long as Rust does not support specialisations.
pub struct InterruptibleSolverWrapper<S: InterruptibleSolver> {
    solver: S,
}

impl<S: InterruptibleSolver> From<S> for InterruptibleSolverWrapper<S> {
    fn from(solver: S) -> Self {
        InterruptibleSolverWrapper { solver }
    }
}

impl<S: InterruptibleSolver> Solver for InterruptibleSolverWrapper<S> {
    fn solve(&self, formula: &CNF) -> SATSolution {
        block_on(self.solver.solve_interruptible(formula))
    }
}

pub struct FlagWaiter {
    flag: Arc<AtomicBool>,
    handle: Option<JoinHandle<SATSolution>>,
}

impl FlagWaiter {
    pub fn start(func: impl FnOnce(Arc<AtomicBool>) -> SATSolution+Send+Sync+'static) -> FlagWaiter {
        let flag = Arc::new(AtomicBool::new(false));
        let other = flag.clone();
        FlagWaiter {
            handle: Some(spawn(move || func(other))),
            flag
        }
    }
}

impl Drop for FlagWaiter {
    fn drop(&mut self) {
        self.flag.store(true, Ordering::Relaxed);
    }
}

impl Future for FlagWaiter {
    type Output = SATSolution;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.flag.load(Ordering::Relaxed) {
            Poll::Ready(self.handle
                .take()
                .expect("FlagWaiter was polled twice!")
                .join()
                .unwrap())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
