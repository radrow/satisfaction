#[macro_use] extern crate async_trait;

/// The CNF representation of a formula
pub mod cnf;
/// The Solver trait which has to be implemented by each solver
pub mod sat_solver;
/// Module containing the reference solver cadical.
mod cadical_solver;
/// Module that contains the custom DPLL solver
mod dpll;
/// Module that specifies the output of a solver
mod sat_solution;
pub mod bruteforce;
/// Branching heuristics one can choose from to customize the [`SatisfactionSolver`].
mod branching_strategy;
/// A module which offers some additional solver,
/// for one that can be interrupted or timed.
pub mod solvers;

pub use cnf::{CNFClause, CNFVar, CNF};
pub use sat_solver::Solver;
pub use cadical_solver::CadicalSolver;
pub use bruteforce::Bruteforce;
pub use branching_strategy::{BranchingStrategy, NaiveBranching, DLIS, DLCS, JeroslawWang, MOM};
pub use dpll::SatisfactionSolver;
pub use sat_solution::{SATSolution, Valuation};
