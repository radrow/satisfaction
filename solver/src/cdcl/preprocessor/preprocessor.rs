use crate::{CNF, SATSolution};

pub trait Preprocessor {
    fn preprocess(&mut self, cnf: &CNF) -> CNF;
    fn restore(&mut self, assignment: SATSolution) -> SATSolution;
}



