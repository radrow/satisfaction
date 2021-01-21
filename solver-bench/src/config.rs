use solver::sat_solver::Solver;
use std::path::PathBuf;

pub struct Config {
    pub input:       String,
    pub plot:        bool,
    pub solvers:     Vec<(String, Box<dyn Solver>)>,
    pub output:      PathBuf,
}
