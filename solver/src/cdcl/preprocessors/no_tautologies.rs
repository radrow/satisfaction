use super::{Preprocessor, PreprocessorFactory};
use crate::{CNF, CNFClause, SATSolution};

pub struct RemoveTautologyInstance;

impl Preprocessor for RemoveTautologyInstance {
    fn preprocess(&mut self, cnf: CNF) -> CNF {
        CNF {
            clauses: cnf.clauses.clone().into_iter().filter(|clause| !is_tautology(clause)).collect(),
            num_variables: cnf.num_variables
        }
    }

    fn restore(&mut self, assignment: SATSolution) -> SATSolution {
        assignment
    }
}

fn is_tautology(clause: &CNFClause) -> bool {
    for variable in &clause.vars {
        if clause.vars.contains(&-(*variable)) {
            return true;
        } 
    }
    false
}

pub struct RemoveTautology;

impl PreprocessorFactory for RemoveTautology {
    fn new(&self) -> Box<dyn Preprocessor> {
        Box::new(RemoveTautologyInstance)
    }
}
