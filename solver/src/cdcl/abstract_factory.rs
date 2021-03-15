use auto_impl::auto_impl;
use super::{
    clause::Clauses,
    variable::Variables,
};

#[auto_impl(Box)]
pub trait AbstractFactory<P> {
    fn create(&self, clauses: &Clauses, variables: &Variables) -> Box<P>;
}
