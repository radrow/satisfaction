/// The field module holdes the current state of the puzzle and can load new states from a file.
mod field;
/// The puzzle creation module allows the creation of a new random puzzle.
mod puzzle_creation;
/// The sat conversion module is used to transform the puzzle into a formula that can be solved by a sat solver
pub mod sat_conversion;

pub use field::{CellType, Field};
pub use puzzle_creation::create_random_puzzle;
pub use sat_conversion::field_to_cnf;
