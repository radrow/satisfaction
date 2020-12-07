mod field;
mod game;
mod field_widget;
mod message;
mod control_widget;
mod puzzle_creation;

use puzzle_creation::create_random_puzzle;
use game::Game;

use iced::{Application, Settings};

fn main() -> iced::Result {
    Game::run(Settings::default())
}