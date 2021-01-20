use solver::BranchingStrategy;
use solver::sat_solver::Solver;

pub struct Config {
    pub input:       String,
    pub plot:        bool,
    pub solvers:     Vec<(String, Box<dyn Solver>)>,
}
