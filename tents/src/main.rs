#[macro_use] extern crate lazy_static;

mod field;
mod game;
mod widgets;
mod message;

use iced::{Settings, Application};
use game::Game;
use solver::solvers::InterruptibleSolver;
use solver::{SatisfactionSolver, DLCS, DLIS, JeroslawWang, MOM};
use std::collections::HashMap;

fn main() -> iced::Result {
    let mut solvers = HashMap::<&'static str, Box<dyn InterruptibleSolver>>::new();
    solvers.insert("Satisfaction (DLIS)", Box::new(SatisfactionSolver::new(DLIS)));
    solvers.insert("Satisfaction (DLCS)", Box::new(SatisfactionSolver::new(DLCS)));
    solvers.insert("Satisfaction (MOM)", Box::new(SatisfactionSolver::new(MOM)));
    solvers.insert("Satisfaction (JeroslawWang)", Box::new(SatisfactionSolver::new(JeroslawWang)));
    Game::run(Settings::with_flags(solvers))
}
