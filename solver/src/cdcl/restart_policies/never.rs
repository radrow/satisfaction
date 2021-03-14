use super::{
    RestartPolicy,
    super::{
        clause::Clauses,
        update::Update,
        variable::Variables,
        abstract_factory::AbstractFactory,
    },
};

pub struct RestartNeverInstance;

impl Update for RestartNeverInstance {}

impl RestartPolicy for RestartNeverInstance {
    fn restart(&mut self) -> bool {false}
}

pub struct RestartNever;

impl AbstractFactory for RestartNever {
    type Product = RestartNeverInstance;
    fn create(&self, _clauses: &Clauses, _variables: &Variables) -> Self::Product {
        RestartNeverInstance
    }
}
