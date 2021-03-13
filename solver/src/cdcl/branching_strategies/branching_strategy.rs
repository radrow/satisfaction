use super::super::{
    variable::Variables,
    clause::Clauses,
    update::{Initialisation, Update},
};
use crate::CNFVar;

pub trait BranchingStrategy: Initialisation+Update {
    fn pick_literal(&mut self, clauses: &Clauses, variables: &Variables) -> Option<CNFVar>;
}
