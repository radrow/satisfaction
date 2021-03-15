use crate::cdcl::{
    clause::{ClauseId, Clauses},
    deletion_strategies::{ClauseDeletionStrategy, ClauseDeletionStrategyFactory},
    update::Update,
    variable::Variables,
};


pub struct NoDeletionInstance;

impl Update for NoDeletionInstance {}

impl ClauseDeletionStrategy for NoDeletionInstance {
    fn delete_clause(&mut self, _clauses: &Clauses, _variables: &Variables) -> Vec<ClauseId> { Vec::new() }
}

pub struct NoDeletion;

impl ClauseDeletionStrategyFactory for NoDeletion {
    fn create(&self, _clauses: &Clauses, _variables: &Variables) -> Box<dyn ClauseDeletionStrategy> {
        Box::new(NoDeletionInstance)
    }
}
