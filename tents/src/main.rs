mod field;
mod game;
mod field_widget;
mod message;
mod control_widget;
mod random_creation_widget;
mod puzzle_creation;
mod number_input;
mod cnf;

use iced::{Settings, Application};
use game::Game;

fn main() -> iced::Result {
    Game::run(Settings::default())
}
