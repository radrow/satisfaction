use super::super::{
    variable::Variables,
    clause::Clauses,
    update::Update,
};
use crate::CNFVar;

pub trait BranchingStrategy: Update {
    fn pick_literal(&mut self, clauses: &Clauses, variables: &Variables) -> Option<CNFVar>;
}
