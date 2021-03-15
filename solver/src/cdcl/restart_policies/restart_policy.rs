use auto_impl::auto_impl;
use crate::cdcl::update::Update;

#[auto_impl(Box)]
pub trait RestartPolicy: Update {
    fn restart(&mut self) -> bool;
}

#[auto_impl(Box)]
pub trait RestartPolicyFactory {
    fn create(&self) -> Box<dyn RestartPolicy>;
}
