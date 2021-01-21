use std::{
    time::Duration,
    sync::{mpsc::channel,Arc},
    thread::spawn,
};
use crate::{CNF, Solver, SATSolution, TimedSolver};

struct SolverWrapper<S>(S);

unsafe impl<S:Solver> Send for SolverWrapper<S> {}
unsafe impl<S:Solver> Sync for SolverWrapper<S> {}


pub struct TimeLimitedSolver<S: Solver> {
    solver: Arc<SolverWrapper<S>>,
    max_duration: Duration,
}

impl<S: Solver + 'static> Solver for TimeLimitedSolver<S> {
    fn solve(&self, formula: &CNF) -> SATSolution {
        let (sender, recv) = channel();
        let solver = self.solver.clone();
        let cloned = formula.clone();
        spawn(move || {
            let solution = solver.0.solve(&cloned);
            let _ = sender.send(solution).unwrap();
        });
        recv.recv_timeout(self.max_duration)
            .unwrap_or(SATSolution::Unknown)
    }
}

impl<S: Solver> TimeLimitedSolver<S> {
    pub fn new(solver: S, max_duration: Duration) -> TimeLimitedSolver<S> {
        TimeLimitedSolver {
            solver: Arc::new(SolverWrapper(solver)),
            max_duration,
        }
    }
}

impl<S: Solver+'static> TimeLimitedSolver<TimedSolver<S>> {
    pub fn solve_timed(&self, formula: &CNF) -> (Duration, SATSolution) {
        let (sender, recv) = channel();
        let solver = self.solver.clone();
        let cloned = formula.clone();
        spawn(move || {
            let solution = solver.0.solve_timed(&cloned);
            let _ = sender.send(solution).unwrap();
        });
        recv.recv_timeout(self.max_duration)
            .unwrap_or((self.max_duration, SATSolution::Unknown))
    }
}
