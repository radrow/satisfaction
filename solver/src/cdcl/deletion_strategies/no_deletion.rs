use crate::cdcl::clause::{ClauseId, Clauses};
use crate::cdcl::deletion_strategies::deletion_strategy::ClauseDeletionStrategy;
use crate::cdcl::update::{Initialisation, Update};
use crate::cdcl::variable::Variables;

pub struct IdentityDeletionStrategy;

impl Initialisation for IdentityDeletionStrategy {
    fn initialise(_clauses: &Clauses, _variables: &Variables) -> Self where Self: Sized { IdentityDeletionStrategy }
}

impl Update for IdentityDeletionStrategy {}

impl ClauseDeletionStrategy for IdentityDeletionStrategy {
    fn delete_clause(&mut self, _clauses: &Clauses, _variables: &Variables) -> Vec<ClauseId> { Vec::new() }
}
