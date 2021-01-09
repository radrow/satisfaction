use solver::sat_solver::Solver;

pub struct Config<'a> {
    pub input:       Option<String>,
    pub return_code: bool,
    pub plot:        bool,
    pub algorithm:   &'a dyn Solver
}
