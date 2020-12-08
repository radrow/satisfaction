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
// fn main () {
//     let game_field: GameField = GameField::new("./maps/tents-8x8-e1.txt");
//     let coordinates: &HashSet<(usize, usize)> = &game_field.tent_coordinates;
//     let width: usize = game_field.width;
//     let height: usize = game_field.height;
//     let col_tent_count: &Vec<usize> = &game_field.col_tent_count;
//     let rows_tent_count: &Vec<usize> = &game_field.rows_tent_count;

//     // print!("coordinates: {:?}\nwidth: {}\nheight: {}\ncol tent count: {:?}\nrows tent count: {:?}", coordinates, width, height, col_tent_count, rows_tent_count);

//     let lol = game_field.to_formula().to_cnf();
//     print!("\n{}", lol.to_dimacs())
// }
