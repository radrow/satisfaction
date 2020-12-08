mod field;
mod game;
mod field_widget;
mod message;
mod control_widget;
mod cnf;
mod formula;

use game::Game;

use iced::{Application, Settings};

fn main() -> iced::Result {
    Game::run(Settings::default())
}