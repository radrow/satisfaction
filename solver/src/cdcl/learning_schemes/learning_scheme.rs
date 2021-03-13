use super::super::{
    variable::Variables,
    clause::{ClauseId, Clauses},
    update::{Initialisation, Update},
};
use crate::{CNFVar, CNFClause};

pub trait LearningScheme: Initialisation+Update {
    /// Finds the a cut in the implication graph.
    fn find_conflict_clause(&mut self, empty_clause: ClauseId, branching_depth: usize, clauses: &Clauses, variables: &Variables) -> Option<(CNFClause, CNFVar, usize)>;
}
