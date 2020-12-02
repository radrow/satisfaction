use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::io;
use std::fs;


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
        let contents: String = fs::read_to_string(path).expect("something went wrong");
        let mut split = contents.split(|c| c == '\n' || c == ' ');

        let width: usize = split.next().unwrap().parse::<usize>().unwrap();
        let height: usize = split.next().unwrap().parse::<usize>().unwrap();

        let mut field: Vec<String> = Vec::new();
        let mut row_counts: Vec<usize> = Vec::new();
        for _ in 0..height {
            field.push(split.next().unwrap().to_string());
            row_counts.push(split.next().unwrap().parse::<usize>().unwrap());
        }
        let column_counts: Vec<usize> = split.map(|x| x.parse::<usize>().unwrap()).collect();

        let mut cells: Vec<Vec<CellType>> = Vec::new();
        for row in field {
            let mut rows: Vec<CellType> = Vec::new();
            for character in row.chars() {
                if character == 'T' {
                    rows.push(CellType::Tree);
                } else {
                    rows.push(CellType::Meadow);
                }
            }
            cells.push(rows);
        }

        Ok(Field {
            row_counts,
            column_counts,
            cells
        })
    }
    
    pub fn tent_coordinates(&self) -> HashSet<(usize, usize)> {
        let mut tent_coordinates: HashSet<(usize, usize)> = HashSet::new();
        let width = self.cells.len();
        let height = self.cells[0].len();
        for (x, row) in self.cells.iter().enumerate() {
            for (y, cell) in row.iter().enumerate() {
                if cell == &CellType::Tree {
                    let left = x as isize - 1;
                    let right = x + 1;
                    let top = y as isize - 1;
                    let bottom = y + 1;

                    if left >= 0 {
                        tent_coordinates.insert((left as usize, y));
                    }
                    if right < width {
                        tent_coordinates.insert((right, y));
                    }
                    if top >= 0 {
                        tent_coordinates.insert((x, top as usize));
                    }
                    if bottom < height {
                        tent_coordinates.insert((x, bottom));
                    }
                }
            }
        }
        tent_coordinates
    }
}

pub fn solve_puzzle(tents: &HashSet<(usize, usize)>, row_counts: &Vec<usize>, column_counts: &Vec<usize>) {
    unimplemented!();
}