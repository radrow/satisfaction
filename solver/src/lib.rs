#[macro_use] extern crate async_trait;

/// The CNF representation of a formula
pub mod cnf;
/// The Solver trait which has to be implemented by each solver
pub mod sat_solver;
mod cadical_solver;
mod dpll;
mod sat_solution;
pub mod bruteforce;
mod branching_strategy;
pub mod solvers;

pub use cnf::{CNFClause, CNFVar, CNF};
pub use sat_solver::Solver;
pub use cadical_solver::CadicalSolver;
pub use bruteforce::Bruteforce;
pub use branching_strategy::{BranchingStrategy, NaiveBranching, DLIS, DLCS, JeroslawWang, MOM};
pub use dpll::SatisfactionSolver;
pub use sat_solution::{SATSolution, Valuation};
