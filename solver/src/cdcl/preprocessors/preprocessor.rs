use crate::{CNF, SATSolution};

#[auto_impl(Box)]
pub trait Preprocessor: Send+Sync {
    fn preprocess(&mut self, cnf: CNF) -> CNF;
    fn restore(&mut self, assignment: SATSolution) -> SATSolution;
}

#[auto_impl(Box)]
pub trait PreprocessorFactory {
    fn new(&self) -> Box<dyn Preprocessor>;
}
