#[macro_use] extern crate lazy_static;

mod field;
mod game;
mod field_widget;
mod message;
mod control_widget;
mod random_creation_widget;
mod puzzle_creation;
mod number_input;
mod cnf;
mod log_widget;

use iced::{Settings, Application};
use game::Game;

fn main() -> iced::Result {
    Game::run(Settings::default())
}
