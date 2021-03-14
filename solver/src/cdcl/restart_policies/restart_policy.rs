use crate::cdcl::update::Update;

pub trait RestartPolicy: Update {
    fn restart(&mut self) -> bool;
}
