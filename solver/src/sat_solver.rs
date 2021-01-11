use crate::cnf::CNF;

pub type Assignment = Vec<bool>;

pub trait Solver {
    fn solve(&self, formula: CNF, num_variables: usize) -> Option<Assignment>;
}
