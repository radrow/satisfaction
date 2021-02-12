#[macro_use]
extern crate lazy_static;

mod field;
/// The game module contains the actual application,
/// i.e. combining view and model, handling user input
/// and events from asynchronous computations
mod game;
/// Messages are used to handle asynchronous computations
/// and user interaction.
/// Any important events that can appear are abstracted by this module.
mod message;
/// All widgets for the graphical representation of the Tents game
/// and user interaction
/// are gathered in this module.
mod widgets;

use game::{Config, Game};
use iced::{Application, Length, Settings};
use solver::solvers::InterruptibleSolver;
use solver::{CadicalSolver, JeroslawWang, SatisfactionSolver, DLCS, DLIS, MOM};
use std::collections::HashMap;

#[doc(hidden)]
fn main() -> iced::Result {
    // Prepare all available solvers
    let mut solvers = HashMap::<&'static str, Box<dyn InterruptibleSolver>>::new();
    solvers.insert("DLIS", Box::new(SatisfactionSolver::new(DLIS)));
    solvers.insert("DLCS", Box::new(SatisfactionSolver::new(DLCS)));
    solvers.insert("MOM", Box::new(SatisfactionSolver::new(MOM)));
    solvers.insert(
        "JeroslawWang",
        Box::new(SatisfactionSolver::new(JeroslawWang)),
    );
    solvers.insert("CadicalSolver", Box::new(CadicalSolver));

    // Fundamental configuration for gui
    let config = Config {
        cell_size: Length::Units(15),
        cell_spacing: 2,
        count_font_size: 12,
        log_field_ratio: (1, 8),
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
