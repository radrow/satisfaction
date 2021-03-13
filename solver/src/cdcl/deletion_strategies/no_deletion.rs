use crate::cdcl::{
    clause::{ClauseId, Clauses},
    deletion_strategies::deletion_strategy::ClauseDeletionStrategy,
    update::Update,
    variable::Variables,
    abstract_factory::AbstractFactory,
};


pub struct NoDeletionInstance;

impl Update for NoDeletionInstance {}

impl ClauseDeletionStrategy for NoDeletionInstance {
    fn delete_clause(&mut self, _clauses: &Clauses, _variables: &Variables) -> Vec<ClauseId> { Vec::new() }
}

pub struct NoDeletion;

impl AbstractFactory for NoDeletion {
    type Product = NoDeletionInstance;
    fn create(&self, _clauses: &Clauses, _variables: &Variables) -> Self::Product {
        NoDeletionInstance
    }
}
