use crate::{CNF, Assignment};


pub trait Solver {
    fn solve(&self, formula: CNF) -> Assignment;
}
