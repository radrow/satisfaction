use crate::{CNF, Assignment};


pub trait Solver {
    fn solve(&self, formula: CNF, num_variables: usize) -> Assignment;
}
