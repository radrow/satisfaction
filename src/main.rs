mod field;
mod game;
mod field_widget;
mod message;
mod control_widget;
mod random_creation_widget;
mod cnf;
mod formula;
mod puzzle_creation;
mod number_input;

use game::Game;

use iced::{Application, Settings};

fn main() -> iced::Result {
    let v = vec![1,2,3,4];
    //field::Field::axis_constraint(&v, 2);
    //Ok(())
    Game::run(Settings::default())
}