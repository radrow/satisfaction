use super::{
    BranchingStrategy,
    super::{
        clause::Clauses,
        variable::Variables,
        update::Update,
        abstract_factory::AbstractFactory,
    },
};
use crate::CNFVar;

pub struct NaiveInstance;

impl Update for NaiveInstance {}

impl BranchingStrategy for NaiveInstance {
    fn pick_literal(&mut self, _clauses: &Clauses, variables: &Variables) -> Option<CNFVar> {
        variables.iter()
            .enumerate()
            .find_map(|(id, var)| if var.assignment.is_none() {
                Some(CNFVar::new(id, true))
            } else {
                None
            })
    }
}

pub struct Naive;

impl AbstractFactory for Naive {
    type Product = NaiveInstance;
    fn create(&self, _clauses: &Clauses, _variables: &Variables) -> Self::Product {
        NaiveInstance
    }
}
