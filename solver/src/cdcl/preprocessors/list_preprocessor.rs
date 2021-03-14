use super::{Preprocessor, PreprocessorFactory};
use crate::{CNF, SATSolution};


pub struct ListPreprocessorInstance {
    preprocessors: Vec<Box<dyn Preprocessor>>,
}

impl Preprocessor for ListPreprocessorInstance {
    fn preprocess(&mut self, cnf: CNF) -> CNF {
        self.preprocessors.iter_mut()
            .fold(cnf, |acc, p| p.preprocess(acc))
    }

    fn restore(&mut self, assignment: SATSolution) -> SATSolution {
        self.preprocessors.iter_mut()
            .rev()
            .fold(assignment, |acc, p| p.restore(acc))
    }
}


pub struct ListPreprocessor(pub Vec<Box<dyn PreprocessorFactory>>);

impl PreprocessorFactory for ListPreprocessor {
    fn new(&self) -> Box<dyn Preprocessor> {
        Box::new(ListPreprocessorInstance {
            preprocessors: self.0.iter()
                .map(|p| p.new())
                .collect()
        })
    }
}
