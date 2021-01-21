use solver::TimeLimitedSolver;
use std::path::PathBuf;

pub struct Config {
    pub input:          String,
    pub solvers:        Vec<(String, Box<dyn TimeLimitedSolver>)>,
    pub output:         PathBuf,
    pub max_duration:   u64,
}
