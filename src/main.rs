mod field;
mod game;

use game::Game;

use iced::{Application, Settings};

fn main() -> iced::Result {
    Game::run(Settings::default())
}
