use crate::cdcl::clause::{ClauseId, Clauses};
use crate::cdcl::update::{Initialisation, Update};
use crate::cdcl::variable::Variables;

pub trait ClauseDeletionStrategy: Initialisation+Update {
    fn delete_clause(&mut self, clauses: &Clauses, variables: &Variables) -> Vec<ClauseId>;
}
