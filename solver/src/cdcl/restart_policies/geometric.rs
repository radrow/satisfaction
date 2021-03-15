use super::{
    RestartPolicy,
    RestartPolicyFactory,
    super::{
        clause::{ClauseId, Clauses},
        update::Update,
        variable::Variables,
    },
};

pub struct RestartGeomInstance { conflicts: u64, rate: u64, factor_percent: u64 }

impl Update for RestartGeomInstance {
    fn on_conflict(&mut self, _empty_clause: ClauseId, _clauses: &Clauses, _variables: &Variables) {
        self.conflicts += 1;
    }
}

impl RestartPolicy for RestartGeomInstance {
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

pub struct RestartGeom {
    rate: u64,
    factor_percent: u64,
}

impl RestartPolicyFactory for RestartGeom {
    fn create(&self) -> Box<dyn RestartPolicy> {
        Box::new(RestartGeomInstance {
            conflicts: 0,
            rate: self.rate,
            factor_percent: self.factor_percent,
        })
    }
}

impl Default for RestartGeom {
    fn default() -> Self {
        RestartGeom {
            rate: 100,
            factor_percent: 150,
        }
    }
}
