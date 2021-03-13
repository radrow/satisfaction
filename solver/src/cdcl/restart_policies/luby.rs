use crate::cdcl::clause::{ClauseId, Clauses};
use crate::cdcl::restart_policies::restart_policy::RestartPolicy;
use crate::cdcl::update::{Initialisation, Update};
use crate::cdcl::variable::Variables;

pub struct RestartLuby { conflicts: u64, rate: u64, luby_state: (u64, u64, u64) }

impl RestartLuby {
    fn next_luby(&mut self) -> u64 {
        let (u, v, w) = self.luby_state;
        self.luby_state =
            if u == w {
                if u == v {
                    (u + 1, 1, w * 2)
                } else {
                    (u, v + 1, w)
                }
            } else {
                (u + 1, v, w)
            };
        v
    }

    pub fn new() -> RestartLuby {
        RestartLuby {
            conflicts: 0,
            rate: 1,
            luby_state: (2, 1, 2), // first step already made
        }
    }
}

impl Update for RestartLuby {
    fn on_conflict(&mut self, _empty_clause: ClauseId, _clauses: &Clauses, _variables: &Variables) {
        self.conflicts += 1;
    }
}

impl Initialisation for RestartLuby {
    fn initialise(_clauses: &Clauses, _variables: &Variables) -> RestartLuby {
        RestartLuby::new()
    }
}

impl RestartPolicy for RestartLuby {
    fn restart(&mut self) -> bool {
        if self.conflicts > self.rate {
            self.rate = 32 * self.next_luby();
            self.conflicts = 0;
            return true
        }
        false
    }
}
