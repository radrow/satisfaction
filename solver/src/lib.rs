#[macro_use]
extern crate async_trait;

/// Branching heuristics one can choose from to customize the [`SatisfactionSolver`].
mod branching_strategy;
mod deletion_strategy;
pub mod bruteforce;
/// Module containing the reference solver cadical.
mod cadical_solver;
/// The CNF representation of a formula
pub mod cnf;
/// Module that contains the custom DPLL solver
mod dpll;
/// Module that specifies the output of a solver
mod sat_solution;
/// The Solver trait which has to be implemented by each solver
pub mod sat_solver;
/// A module which offers some additional solver,
/// for one that can be interrupted or timed.
pub mod solvers;

pub use branching_strategy::{BranchingStrategy, JeroslawWang, NaiveBranching, DLCS, DLIS, MOM};
pub use bruteforce::Bruteforce;
pub use cadical_solver::CadicalSolver;
pub use cnf::{CNFClause, CNFVar, CNF};
pub use dpll::SatisfactionSolver;
pub use sat_solution::{SATSolution, Valuation};
pub use sat_solver::Solver;
