#[macro_use] extern crate lazy_static;

mod field;
mod game;
mod widgets;
mod message;

use iced::{Settings, Application, Length};
use game::{Game, Config};
use solver::solvers::InterruptibleSolver;
use solver::{SatisfactionSolver, DLCS, DLIS, JeroslawWang, MOM, CadicalSolver};
use std::collections::HashMap;

fn main() -> iced::Result {
    let mut solvers = HashMap::<&'static str, Box<dyn InterruptibleSolver>>::new();
    solvers.insert("DLIS", Box::new(SatisfactionSolver::new(DLIS)));
    solvers.insert("DLCS", Box::new(SatisfactionSolver::new(DLCS)));
    solvers.insert("MOM", Box::new(SatisfactionSolver::new(MOM)));
    solvers.insert("JeroslawWang", Box::new(SatisfactionSolver::new(JeroslawWang)));
    solvers.insert("CadicalSolver", Box::new(CadicalSolver));

    let config = Config {
        cell_size: Length::Units(15),
        cell_spacing: 2,
        count_font_size: 12,
        log_field_ratio: (1,8),
        control_field_ratio: (2, 8),
        spacing: 5,
        padding: 10,
        button_font_size: 12,
        log_font_size: 10,
        scrollbar_width: 4,
        scrollbar_margin: 4,
        solvers,
    };
    Game::run(Settings::with_flags(config))
}
