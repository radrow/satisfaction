use crate::cdcl::update::{Initialisation, Update};

pub trait RestartPolicy: Initialisation+Update {
    fn restart(&mut self) -> bool;
}
