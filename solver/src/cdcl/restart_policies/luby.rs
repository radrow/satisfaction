use super::{
    RestartPolicy,
    RestartPolicyFactory,
    super::{
        clause::{ClauseId, Clauses},
        update::Update,
        variable::Variables,
    },
};

pub struct RestartLubyInstance { conflicts: u64, rate: u64, luby_state: (u64, u64, u64) }

impl RestartLubyInstance {
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
}

impl Update for RestartLubyInstance {
    fn on_conflict(&mut self, _empty_clause: ClauseId, _clauses: &Clauses, _variables: &Variables) {
        self.conflicts += 1;
    }
}

impl RestartPolicy for RestartLubyInstance {
    fn restart(&mut self) -> bool {
        if self.conflicts > self.rate {
            self.rate = 32 * self.next_luby();
            self.conflicts = 0;
            return true
        }
        false
    }
}

pub struct RestartLuby(u64);

impl RestartPolicyFactory for RestartLuby {
    fn create(&self) -> Box<dyn RestartPolicy> {
        Box::new(RestartLubyInstance {
            conflicts: 0,
            rate: self.0,
            luby_state: (2, 1, 2), // first step already made
        })
    }
}

impl Default for RestartLuby {
    fn default() -> Self {
        RestartLuby(1)
    }
}
