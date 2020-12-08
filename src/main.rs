mod field;
mod game;
mod field_widget;
mod message;
mod control_widget;
mod cnf;
mod formula;
mod puzzle_creation;

use game::Game;

use iced::{Application, Settings};

fn main() -> iced::Result {
    puzzle_creation::create_random_puzzle(4, 4);
    Game::run(Settings::default())
}
