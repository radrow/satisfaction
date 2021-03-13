use std::collections::VecDeque;
use tinyset::SetUsize;
use super::{
    super::{
        variable::{VariableId, Variables, Assignment, AssignmentType},
        clause::{ClauseId, Clauses},
        update::{Initialisation, Update},
    },
    learning_scheme::LearningScheme,
};
use crate::{CNFVar, CNFClause};

pub struct RelSAT;

impl Initialisation for RelSAT {
    fn initialise(_clauses: &Clauses, _variables: &Variables) -> Self where Self: Sized {
        RelSAT
    }
}
impl Update for RelSAT {}


impl LearningScheme for RelSAT {
    fn find_conflict_clause(&mut self, empty_clause: ClauseId, branching_depth: usize, clauses: &Clauses, variables: &Variables) -> Option<(CNFClause, CNFVar, usize)> {
        // Start with vertices that are connected to the conflict clause
        let mut literal_queue: VecDeque<VariableId> = clauses[empty_clause].literals
            .iter()
            .map(|lit| lit.id)
            .collect();
        let mut visited: SetUsize = literal_queue
            .iter()
            .cloned()
            .collect();


        let mut clause = CNFClause::with_capacity(literal_queue.len());
        let mut assertion_literal = None;
        let mut assertion_level = 0;

        while let Some(id) = literal_queue.pop_front() {
            let variable = &variables[id];
            match variable.assignment {
                // For each forced literal of current branching_level
                // append connected vertices to literal queue
                Some(Assignment{branching_level, reason: AssignmentType::Forced(reason), ..}) if branching_level == branching_depth => { 
                    for lit in clauses[reason].literals.iter() {
                        if visited.insert(lit.id) {
                            literal_queue.push_back(lit.id);
                        }
                    }
                },
                Some(Assignment{sign, branching_level, ..}) => {
                    let literal = CNFVar::new(id, !sign);
                    clause.push(literal);
                    if branching_level != branching_depth {
                        assertion_level = std::cmp::max(assertion_level, branching_level);
                    } else {
                        let last_index = clause.len()-1;
                        clause.vars.swap(0, last_index);
                        assertion_literal = Some(literal);
                    }
                }
                _ => {},
            }
        }
        assertion_literal.map(|literal| (clause, literal, assertion_level))
    }
}
