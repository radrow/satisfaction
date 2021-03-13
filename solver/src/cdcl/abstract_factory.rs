use super::{
    clause::Clauses,
    variable::Variables,
};

pub trait AbstractFactory {
    type Product: Sized;

    fn create(&self, clauses: &Clauses, variables: &Variables) -> Self::Product;
}
