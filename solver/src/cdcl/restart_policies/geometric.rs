use crate::cdcl::clause::{ClauseId, Clauses};
use crate::cdcl::restart_policies::restart_policy::RestartPolicy;
use crate::cdcl::update::{Initialisation, Update};
use crate::cdcl::variable::Variables;

pub struct RestartGeom { conflicts: u64, rate: u64, factor_percent: u64 }

impl RestartGeom {
    pub fn new(rate: u64, factor_percent: u64) -> RestartGeom {
        RestartGeom{
            conflicts: 0,
            rate,
            factor_percent,
        }
    }
}

impl Update for RestartGeom {
    fn on_conflict(&mut self, _empty_clause: ClauseId, _clauses: &Clauses, _variables: &Variables) {
        self.conflicts += 1;
    }
}

impl Initialisation for RestartGeom {
    fn initialise(_clauses: &Clauses, _variables: &Variables) -> RestartGeom {
        RestartGeom::new(100, 150)
    }
}

impl RestartPolicy for RestartGeom {
    fn restart(&mut self) -> bool {
        if self.conflicts > self.rate {
            self.rate *= self.factor_percent;
            self.rate /= 100;
            self.conflicts = 0;
            return true
        }
        false
    }
}
