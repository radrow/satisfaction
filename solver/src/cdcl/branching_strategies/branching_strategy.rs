use auto_impl::auto_impl;
use super::super::{
    variable::Variables,
    clause::Clauses,
    update::Update,
};
use crate::CNFVar;

#[auto_impl(Box)]
pub trait BranchingStrategy: Update {
    fn pick_literal(&mut self, clauses: &Clauses, variables: &Variables) -> Option<CNFVar>;
}

#[auto_impl(Box)]
pub trait BranchingStrategyFactory {
    fn create(&self, clauses: &Clauses, variables: &Variables) -> Box<dyn BranchingStrategy>;
}
