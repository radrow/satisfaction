use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::io;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CellType {
    Unknown,
    Tent,
    Tree,
    Meadow,
}

pub struct Field {
    pub cells: Vec<Vec<CellType>>,
    pub row_counts: Vec<usize>,
    pub column_counts: Vec<usize>,
}

impl Field {
    pub fn from_file(path: &Path) -> io::Result<Field> {
        unimplemented!();
    }
}

pub fn solve_puzzle(tents: &HashSet<(usize, usize)>, row_counts: &Vec<usize>, column_counts: &Vec<usize>) {
    unimplemented!();
}