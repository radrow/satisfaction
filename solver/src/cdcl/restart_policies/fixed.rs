use crate::cdcl::clause::{ClauseId, Clauses};
use crate::cdcl::restart_policies::restart_policy::RestartPolicy;
use crate::cdcl::update::{Initialisation, Update};
use crate::cdcl::variable::Variables;

pub struct RestartFixed { conflicts: u64, rate: u64 }

impl RestartFixed {
    pub fn new(rate: u64) -> RestartFixed {
        RestartFixed{
            conflicts: 0,
            rate,
        }
    }
}

impl Update for RestartFixed {
    fn on_conflict(&mut self, _empty_clause: ClauseId, _clauses: &Clauses, _variables: &Variables) {
        self.conflicts += 1;
    }
}

impl Initialisation for RestartFixed {
    fn initialise(_clauses: &Clauses, _variables: &Variables) -> RestartFixed {
        RestartFixed::new(550)
    }
}

impl RestartPolicy for RestartFixed {
    fn restart(&mut self) -> bool {
        if self.conflicts > self.rate {
            self.conflicts = 0;
            return true
        }
        false
    }
}
