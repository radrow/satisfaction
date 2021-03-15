use super::{
    RestartPolicy,
    RestartPolicyFactory,
    super::{
        clause::Clauses,
        update::Update,
        variable::Variables,
    },
};

pub struct RestartNeverInstance;

impl Update for RestartNeverInstance {}

impl RestartPolicy for RestartNeverInstance {
    fn restart(&mut self) -> bool {false}
}

pub struct RestartNever;

impl RestartPolicyFactory for RestartNever {
    fn create(&self) -> Box<dyn RestartPolicy> {
        Box::new(RestartNeverInstance)
    }
}
