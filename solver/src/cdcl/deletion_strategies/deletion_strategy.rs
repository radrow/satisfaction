use auto_impl::auto_impl;
use crate::cdcl::clause::{ClauseId, Clauses};
use crate::cdcl::update::Update;
use crate::cdcl::variable::Variables;

#[auto_impl(Box)]
pub trait ClauseDeletionStrategy: Update {
    fn delete_clause(&mut self, clauses: &Clauses, variables: &Variables) -> Vec<ClauseId>;
}

#[auto_impl(Box)]
pub trait ClauseDeletionStrategyFactory {
    fn create(&self, clauses: &Clauses, variables: &Variables) -> Box<dyn ClauseDeletionStrategy>;
}
