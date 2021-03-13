use super::{
    variable::{VariableId, Variables},
    clause::{ClauseId, Clause, Clauses},
};
use crate::CNFVar;

pub trait Update {
    fn on_assign(&mut self, _variable: VariableId, _clauses: &Clauses, _variables: &Variables) {}
    fn on_unassign(&mut self, _literal: CNFVar, _clauses: &Clauses, _variables: &Variables) {}
    fn on_learn(&mut self, _learned_clause: ClauseId, _clauses: &Clauses, _variables: &Variables) {}
    fn on_conflict(&mut self, _empty_clause: ClauseId, _clauses: &Clauses, _variables: &Variables) {}
    fn on_deletion(&mut self, _deleted_clause: &Clause) {}
}

pub trait Initialisation {
    fn initialise(clauses: &Clauses, variables: &Variables) -> Self where Self: Sized;
}
