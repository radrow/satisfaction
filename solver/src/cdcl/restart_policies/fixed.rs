use super::{
    RestartPolicy,
    super::{
        clause::{ClauseId, Clauses},
        update::Update,
        variable::Variables,
        abstract_factory::AbstractFactory,
    },
};

pub struct RestartFixedInstance { conflicts: u64, rate: u64 }

impl Update for RestartFixedInstance {
    fn on_conflict(&mut self, _empty_clause: ClauseId, _clauses: &Clauses, _variables: &Variables) {
        self.conflicts += 1;
    }
}

impl RestartPolicy for RestartFixedInstance {
    fn restart(&mut self) -> bool {
        if self.conflicts > self.rate {
            self.conflicts = 0;
            return true
        }
        false
    }
}

pub struct RestartFixed(pub u64);

impl Default for RestartFixed {
    fn default() -> Self {
        RestartFixed(100)
    }
}

impl AbstractFactory for RestartFixed {
    type Product = RestartFixedInstance;
    fn create(&self, _clauses: &Clauses, _variables: &Variables) -> Self::Product {
        RestartFixedInstance {
            conflicts: 0,
            rate: self.0,
        }
    }
}
