use crate::cdcl::clause::Clauses;
use crate::cdcl::restart_policies::restart_policy::RestartPolicy;
use crate::cdcl::update::{Initialisation, Update};
use crate::cdcl::variable::Variables;

pub struct RestartNever;

impl Initialisation for RestartNever {
    fn initialise(_clauses: &Clauses, _variables: &Variables) -> RestartNever {
        RestartNever
    }
}

impl Update for RestartNever {}

impl RestartPolicy for RestartNever {
    fn restart(&mut self) -> bool {false}
}
