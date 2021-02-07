#[macro_use] extern crate lazy_static;

mod field;
mod game;
mod widgets;
mod message;

use iced::{Settings, Application};
use game::Game;

fn main() -> iced::Result {
    Game::run(Settings::default())
}
