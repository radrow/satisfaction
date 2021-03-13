use crate::cdcl::clause::{ClauseId, Clauses};
use crate::cdcl::update::Update;
use crate::cdcl::variable::Variables;

pub trait ClauseDeletionStrategy: Update {
    fn delete_clause(&mut self, clauses: &Clauses, variables: &Variables) -> Vec<ClauseId>;
}
