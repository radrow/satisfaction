use std::path::Path;
use std::error::Error;

use tokio::fs::read_to_string;

const MIN_WIDTH: usize = 2;
const MAX_WIDTH: usize = 45;
const MIN_HEIGHT: usize = 2;
const MAX_HEIGHT: usize = 35;


/// Coordinates of a tree
pub type TentPlace = (usize, usize);

/// Enum giving detail information about
/// what went wrong during parsing.
#[derive(Debug)]
enum FieldParserError {
    /// The height in the Tent game file is missing.
    HeightNotSpecified,
    /// The width in the Tent game file is missing.
    WidthNotSpecified,
    /// A line specifying a row of the Tents field has not row count.
    MissingRowCount(usize),
    /// There are less or more cells specified in a row than expected.
    ///
    /// # Arguments
    /// * `expected` - The number of cells that was expected, i.e. specified by the file header.
    /// * `found` - The number of cells found.
    /// * `line` - The line of interest.
    WrongNumberOfCells{expected: usize, found: usize, line: usize},
    /// In the last line there are less or more column count specified than expected.
    ///
    /// # Arguments
    /// * `expected` - The count of numbers expected, i.e. specified by the file header.
    /// * `expected` - The count of numbers found.
    MissingColumnCounts{expected: usize, found: usize},
    /// Only numbers and characters 'T' (= Tree) and . (= Meadow/Nothing) are allow.
    /// If a strange character it is noted to the user.
    InvalidCharacter(char),
    /// If a number was expected but could not be found,
    /// the user is informed with the respective line number.
    ParsingNumberFailed(usize),
}

impl std::fmt::Display for FieldParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            FieldParserError::WidthNotSpecified =>
                "A width was expected but not found.".to_string(),
            FieldParserError::HeightNotSpecified =>
                "A height was expected but not found.".to_string(),
            FieldParserError::MissingRowCount(line) =>
                format!("In line {} no tent count was specified.", line),
            FieldParserError::WrongNumberOfCells{line, expected, found} =>
                format!("Not enough cells were specified in line {}: Expected {} but found {}.", line, expected, found),
            FieldParserError::MissingColumnCounts{expected, found} =>
                format!("Could not find enough column counts in last line: Expected {} but found {}.", expected, found),
            FieldParserError::InvalidCharacter(character) =>
                format!("Encountered an invalid character {}.", character),
            FieldParserError::ParsingNumberFailed(line) =>
                format!("Parsing failed in line {}.", line),
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
        Field::from_string(contents)
    }

    pub fn from_string(input: String) -> Result<Field, Box<dyn std::error::Error>> {
        // Only numbers, 'T', '.' and space are allowed.
        if let Some(c) = input.chars()
            .filter(|c| !(c.is_ascii_whitespace() || c.is_numeric() || *c == 'T' || *c == '.'))
            .next() {
            return Err(FieldParserError::InvalidCharacter(c).into());
        };

        let mut lines = input.lines();
        
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
            .ok_or(FieldParserError::ParsingNumberFailed(1))?;

        let width: usize = split.next()
            .ok_or(FieldParserError::WidthNotSpecified)?
            .parse::<usize>()
            .ok()
            .ok_or(FieldParserError::ParsingNumberFailed(1))?;

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
            .ok_or(FieldParserError::ParsingNumberFailed(line_number))?;

        if cells.len() != width {
            return Err(FieldParserError::WrongNumberOfCells{expected: width, found: cells.len(), line: line_number}.into());
        }

        let row = cells.chars()
            .map(|c| {
                match c {
                    'T' => Ok(CellType::Tree),
                    '.' => Ok(CellType::Meadow),
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
                    .ok_or(FieldParserError::ParsingNumberFailed(line_number))
            }).collect::<Result<Vec<usize>, FieldParserError>>()
            .and_then(|vec|{
                if vec.len() == width { Ok(vec) }
                else { Err(FieldParserError::MissingColumnCounts{expected: width, found: vec.len()}) }
            })
            
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


#[cfg(test)]
mod tests {
    use super::*;

    fn compare_messages<T: std::fmt::Debug,E: std::error::Error>(result: Result<T, Box<dyn std::error::Error>>, expected: E) {
        let error = result.unwrap_err();
        assert!(error.to_string() == expected.to_string(), error.to_string());
    }

    #[test]
    fn missing_height() {
        let input = "";
        compare_messages(
            Field::from_string(input.to_string()),
            FieldParserError::HeightNotSpecified,
        );
    }

    #[test]
    fn missing_width() {
        let input = 
r"3
.T. 1
... 0
.T. 1
1 0 1";
        compare_messages(
            Field::from_string(input.to_string()),
            FieldParserError::WidthNotSpecified,
        );
    }

    #[test]
    fn missing_row_count() {
        let input =
r"3 3
.T.
... 0
.T. 1
1 0 1";
        compare_messages(
            Field::from_string(input.to_string()),
            FieldParserError::MissingRowCount(1)
        );
    }

    #[test]
    fn wrong_number_of_cells() {
        let input =
r"3 3
.T. 1
.. 0
.T. 1
1 0";
        compare_messages(
            Field::from_string(input.to_string()),
            FieldParserError::WrongNumberOfCells{expected: 3, found: 2, line: 2}
        );
    }

    #[test]
    fn missing_column_counts() {
        let input =
r"3 3
.T. 1
... 0
.T. 1
1";
        compare_messages(
            Field::from_string(input.to_string()),
            FieldParserError::MissingColumnCounts{expected: 3, found: 1}
        );
    }

    #[test]
    fn invalid_character_x() {
        let input =
r"3 3
.T. x
... 0
.T. 1
1 0 1";
        compare_messages(
            Field::from_string(input.to_string()),
            FieldParserError::InvalidCharacter('x')
        );
    }

    #[test]
    fn parsing_number_failed() {
        let input =
r"T 
.T. 1
... 0
.T. 1
1 0 1";
        compare_messages(
            Field::from_string(input.to_string()),
            FieldParserError::ParsingNumberFailed(1)
        );
    }
}
