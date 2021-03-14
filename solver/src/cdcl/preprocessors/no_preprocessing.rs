use super::{Preprocessor, PreprocessorFactory};
use crate::{CNF, SATSolution};

pub struct NoPreprocessingInstance;

impl Preprocessor for NoPreprocessingInstance {
    fn preprocess(&mut self, cnf: CNF) -> CNF {
        cnf
    }

    fn restore(&mut self, assignment: SATSolution) -> SATSolution {
        assignment
    }
}

pub struct NoPreprocessing;

impl PreprocessorFactory for NoPreprocessing {
    fn new(&self) -> Box<dyn Preprocessor> {
        Box::new(NoPreprocessingInstance)
    }
}
