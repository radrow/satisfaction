use solver::sat_solver::Solver;
use std::path::PathBuf;

pub struct Config {
    pub input:       Option<String>,
    pub return_code: bool,
    pub solver:      Box<dyn Solver>,
    pub output:      Option<PathBuf>,
}
