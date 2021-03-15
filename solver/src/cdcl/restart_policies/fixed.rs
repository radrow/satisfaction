use super::{
    RestartPolicy,
    RestartPolicyFactory,
    super::{
        clause::{ClauseId, Clauses},
        update::Update,
        variable::Variables,
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
        RestartFixed(700)
    }
}

impl RestartPolicyFactory for RestartFixed {
    fn create(&self) -> Box<dyn RestartPolicy> {
        Box::new(RestartFixedInstance {
            conflicts: 0,
            rate: self.0,
        })
    }
}
