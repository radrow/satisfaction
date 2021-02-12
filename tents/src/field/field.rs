use std::path::Path;
use std::error::Error;

use super::sat_conversion;
use solver::sat_solver;

use tokio::fs::read_to_string;

const MIN_WIDTH: usize = 2;
const MAX_WIDTH: usize = 45;
const MIN_HEIGHT: usize = 2;
const MAX_HEIGHT: usize = 35;


/// Coordignates of a tree
pub type TentPlace = (usize, usize);

#[derive(Debug)]
enum FieldParserError {
    WidthNotSpecified,
    HeightNotSpecified,
    MissingRowCount(usize),
    WrongNumberOfCells{expected: usize, found: usize, line: usize},
    MissingColumnCounts{expected: usize, found: usize},
    InvalidCharacter(char),
    ParsingFailed(usize),
}

impl std::fmt::Display for FieldParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            FieldParserError::WidthNotSpecified =>
                "A width was expected but not found".to_string(),
            FieldParserError::HeightNotSpecified =>
                "A height was expected but not found".to_string(),
            FieldParserError::MissingRowCount(line) =>
                format!("In line {} no tent count was specified", line),
            FieldParserError::WrongNumberOfCells{line, expected, found} =>
                format!("Not enough cells were specified in line {}: Expected {} but found {}", line, expected, found),
            FieldParserError::MissingColumnCounts{expected, found} =>
                format!("Could not find enough column counts in last line: Expected {} but found {}", expected, found),
            FieldParserError::InvalidCharacter(character) =>
                format!("Encountered an invalid character {}", character),
            FieldParserError::ParsingFailed(line) =>
                format!("Parsing failed in line {}", line),
        })
    }
}

impl Error for FieldParserError {}

#[derive(Debug)]
pub enum FieldCreationError {
    WidthColumnCountDifference(usize, usize),
    HightRowCountDIfference(usize, usize),
    WidthTooLarge(usize),
    HeightTooLarge(usize),
    UnequalHeight{column: usize, expected: usize, found: usize},
    FieldTooSmall,
}

impl std::fmt::Display for FieldCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            FieldCreationError::WidthColumnCountDifference(width, column_count) =>
                format!("Field width {} and number of column counts {} differ.", width, column_count),
            FieldCreationError::HightRowCountDIfference(height, row_count) => 
                format!("Field height {} and number of row counts {} differ.", height, row_count),
            FieldCreationError::WidthTooLarge(width) => 
                format!("Field width {} is to large. It should be less than {}.", width, MAX_WIDTH),
            FieldCreationError::HeightTooLarge(height) => 
                format!("Field height {} is to large. It should be less than {}.", height, MAX_HEIGHT),
            FieldCreationError::FieldTooSmall => 
                format!("Specified field size was too small. It must be at least {} x {}.", MIN_WIDTH, MIN_HEIGHT),
            FieldCreationError::UnequalHeight{column, expected, found} => 
                format!("In column {} the there are not enough cells specified: Expected {} but found {}.", column, expected, found),
        })
    }
}

impl Error for FieldCreationError {}


/// Datatype that describes the content of a single cell
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CellType {
    Tent,
    Tree,
    Meadow,
}

#[derive(Debug,Clone)] // TODO: Write a appropriate debug printing
/// Representation of the whole puzzle
pub struct Field {
    pub cells: Vec<Vec<CellType>>,
    pub row_counts: Vec<usize>,
    pub column_counts: Vec<usize>,
    pub height: usize,
    pub width: usize,
}


impl Field {
    pub fn new(cells: Vec<Vec<CellType>>, row_counts: Vec<usize>, column_counts: Vec<usize>) -> Result<Field, FieldCreationError> {
        // The field have to be of a certain size, otherwise the Tents game is futile.
        let height = cells.len();
        if height < MIN_HEIGHT { return Err(FieldCreationError::FieldTooSmall) }

        let width = cells[0].len();
        if width < MIN_WIDTH { return Err(FieldCreationError::FieldTooSmall) }

        // Field that are too big, are neither solvable nor presentable via gui.
        if width > MAX_WIDTH  { return Err(FieldCreationError::WidthTooLarge(width)) }
        if height > MAX_HEIGHT  { return Err(FieldCreationError::HeightTooLarge(height)) }

        // Each column vector must have the same size.
        if let Some((column, found)) = cells.iter()
            .enumerate()
            .find_map(|(index, col)| {
                if col.len() != width { Some((index, col.len())) }
                else { None } 
            }) { return Err(FieldCreationError::UnequalHeight{column, expected: height, found}) }

        // The number of column and row counts must be the same as the width and height
        if width != column_counts.len() { return Err(FieldCreationError::WidthColumnCountDifference(width, column_counts.len())) }
        if height != row_counts.len() { return Err(FieldCreationError::HightRowCountDIfference(height, row_counts.len())) }

        Ok(Field {
            cells,
            row_counts,
            column_counts,
            width,
            height,
        })
    }

    /// Function to create an empty `field`.
    /// 
    /// # Arguments
    /// 
    /// * `width` - Width of the puzzle.
    /// * `height` - Height of the puzzle.
    pub fn create_empty(width: usize, height: usize) -> Result<Field, FieldCreationError> {
        let prototype = vec![CellType::Meadow; width];
        let cells = vec![prototype; height];

        let column_counts = vec![0; width];
        let row_counts = vec![0; height];

        Field::new(cells, row_counts, column_counts)
    }

    pub async fn from_file(path: impl AsRef<Path>) -> Result<Field, Box<dyn std::error::Error>> {
        let contents: String = read_to_string(path).await?;

        // Only numbers, 'T', '.' and space are allowed.
        if let Some(c) = contents.chars()
            .filter(|c| !(c.is_ascii_whitespace() || c.is_numeric() || *c == 'T' || *c == '.'))
            .next() {
            return Err(FieldParserError::InvalidCharacter(c).into());
        };

        let mut lines = contents.lines();
        
        let (width, height) = Field::parse_size(
            lines.next()
                .ok_or(FieldParserError::HeightNotSpecified)?
        )?;

        let mut lines_with_numbers = lines.zip(1..);

        let (cells, row_counts) = lines_with_numbers.by_ref()
            .take(height)
            .map(|(line, number)| Field::parse_tent_line(line, number, width))
            .collect::<Result<Vec<_>, FieldParserError>>()?
            .into_iter().unzip();

        let column_counts = lines_with_numbers.next()
            .ok_or(FieldParserError::MissingColumnCounts{expected: width, found: 0})
            .and_then(|(line, number)| Field::parse_column_counts(line, number, width))?;

        Field::new(cells, row_counts, column_counts)
            .map_err(FieldCreationError::into)
    }

    fn parse_size(line: &str) -> Result<(usize, usize), FieldParserError> {
        let mut split = line.split(' ');

        let height: usize = split.next()
            .ok_or(FieldParserError::HeightNotSpecified)?
            .parse::<usize>()
            .ok()
            .ok_or(FieldParserError::ParsingFailed(1))?;

        let width: usize = split.next()
            .ok_or(FieldParserError::WidthNotSpecified)?
            .parse::<usize>()
            .ok()
            .ok_or(FieldParserError::ParsingFailed(1))?;

        Ok((width, height))
    }

    fn parse_tent_line(line: &str, line_number: usize, width: usize) -> Result<(Vec<CellType>, usize), FieldParserError> {
        let mut split = line.splitn(2, ' ');

        let cells = split.next()
            .ok_or(FieldParserError::WrongNumberOfCells{line: line_number, expected: width, found: 0})?;

        let row_count = split.next()
            .ok_or(FieldParserError::MissingRowCount(line_number))?
            .parse::<usize>()
            .ok()
            .ok_or(FieldParserError::ParsingFailed(line_number))?;

        if cells.len() != width {
            return Err(FieldParserError::WrongNumberOfCells{expected: width, found: cells.len(), line: line_number}.into());
        }

        let row = cells.chars()
            .map(|c| {
                match c {
                    'T' => Ok(CellType::Tree),
                    '.' => Ok(CellType::Meadow),
                    // TODO: More precise location
                    _   => Err(FieldParserError::InvalidCharacter(c)),
                }
            }).collect::<Result<Vec<CellType>, FieldParserError>>()?;

        Ok((row, row_count))
    }

    fn parse_column_counts(line: &str, line_number: usize, width: usize) -> Result<Vec<usize>, FieldParserError> {
        line.split(' ')
            .map(|number| {
                number.parse::<usize>()
                    .ok()
                    .ok_or(FieldParserError::ParsingFailed(line_number))
            }).collect::<Result<Vec<usize>, FieldParserError>>()
            .and_then(|vec|{
                if vec.len() == width { Ok(vec) }
                else { Err(FieldParserError::MissingColumnCounts{expected: width, found: vec.len()}) }
            })
            
    }

    pub fn is_solved(&self) -> bool {
        return self.count_constraint_holds() &&
            self.neighbour_constraint_holds() &&
            self.correspondence_constraint_holds()

    }

    pub fn count_constraint_holds(&self) -> bool {
        unimplemented!()
    }

    pub fn neighbour_constraint_holds(&self) -> bool {
        unimplemented!()
    }

    pub fn correspondence_constraint_holds(&self) -> bool {
        let id_mapping = sat_conversion::make_id_mapping(self);

        let (formula, _) = sat_conversion::make_correspondence_constraints(self, &id_mapping);

        let v_size = id_mapping.len();
        let mut valuation = Vec::with_capacity(v_size);
        unsafe { valuation.set_len(v_size); }

        for ((x, y), i) in id_mapping.iter() {
            match self.get_cell(*x, *y) {
                Tent => valuation[*i] = true,
                Meadow => valuation[*i] = false,
                _ => ()
            }
        }

        sat_solver::check_valuation(&formula, &valuation)
    }

    #[inline]
    pub fn get_cell(&self, row: usize, column: usize) -> CellType {
        self.cells[row][column]
    }

    #[allow(dead_code)]
    #[inline]
    pub fn get_cell_mut(&mut self, row: usize, column: usize) -> &mut CellType {
        &mut self.cells[row][column]
    }

    /// Returns a vector of eligible places for a tent
    pub fn tent_coordinates(&self) -> Vec<TentPlace> {
        let mut tents_by_trees = Vec::new();

        for row in 0..self.height {
            for column in 0..self.width {
                if self.get_cell(row, column) == CellType::Tree {
                    if let Some(left) = column.checked_sub(1) {
                        if self.get_cell(row, left) != CellType::Tree {
                            tents_by_trees.push((row, left));
                        }
                    }

                    let right = column + 1;
                    if right < self.width && self.get_cell(row, right) != CellType::Tree {
                        tents_by_trees.push((row, right));
                    }

                    if let Some(top) = row.checked_sub(1) {
                        if self.get_cell(top, column) != CellType::Tree {
                            tents_by_trees.push((top, column));
                        }
                    }

                    let bottom = row + 1;
                    if bottom < self.height && self.get_cell(bottom, column) != CellType::Tree {
                        tents_by_trees.push((bottom, column));
                    }
                }
            }
        }
        tents_by_trees
    }

}
