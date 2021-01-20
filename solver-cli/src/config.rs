use solver::sat_solver::Solver;

pub struct Config {
    pub input:       Option<String>,
    pub return_code: bool,
    pub solver:      Box<dyn Solver>,
}
