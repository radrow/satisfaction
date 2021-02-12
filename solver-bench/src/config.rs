use solver::solvers::InterruptibleSolver;
use std::path::PathBuf;

pub struct Config {
    pub input: String,
    pub solvers: Vec<(String, Box<dyn InterruptibleSolver>)>,
    pub output: PathBuf,
    pub max_duration: u64,
}
