use auto_impl::auto_impl;
use super::super::{
    variable::Variables,
    clause::{ClauseId, Clauses},
    update::Update,
};
use crate::{CNFVar, CNFClause};

#[auto_impl(Box)]
pub trait LearningScheme: Update {
    /// Finds the a cut in the implication graph.
    fn find_conflict_clause(&mut self, empty_clause: ClauseId, branching_depth: usize, clauses: &Clauses, variables: &Variables) -> Option<(CNFClause, CNFVar, usize)>;
}

#[auto_impl(Box)]
pub trait LearningSchemeFactory {
    fn create(&self, clauses: &Clauses, variables: &Variables) -> Box<dyn LearningScheme>;
}
