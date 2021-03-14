use super::{preprocessor::Preprocessor};
use crate::{CNF, CNFClause, SATSolution};

pub struct RemoveTautology;

impl Preprocessor for RemoveTautology {
    fn preprocess(&mut self, cnf: &CNF) -> CNF {
        CNF {
            clauses: cnf.clauses.clone().into_iter().filter(|clause| !is_tautology(clause)).collect(),
            num_variables: cnf.num_variables
        }
    }

    fn restore(&mut self, assignment: SATSolution) -> SATSolution {
        assignment
    }
}

impl RemoveTautology {
    pub fn new() -> RemoveTautology {
        RemoveTautology
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