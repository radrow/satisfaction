mod field;
mod game;
mod field_widget;
mod message;
mod control_widget;

use game::Game;

use iced::{Application, Settings};

fn main() -> iced::Result {
    Game::run(Settings::default())
}