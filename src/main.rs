mod field;
mod game;
mod field_widget;
mod message;
mod control_widget;
mod random_creation_widget;
mod puzzle_creation;
mod number_input;

use game::Game;

use iced::{Application, Settings};

fn main() -> iced::Result {
    Game::run(Settings::default())
}