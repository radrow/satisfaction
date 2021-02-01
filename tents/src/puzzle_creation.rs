use crate::field::*;
use rand::prelude::*;

const TOO_LARGE_FIELD: usize = 400;

/// Function to create a random tent puzzle.
/// Returns a Result of a `Field` or an `String` with an errormessage.
/// 
/// # Arguments
/// 
/// * `height` - Height of the puzzle.
/// * `width` - The width of the puzzle.
/// 
/// # Example
/// 
/// ```
/// // creation of a 8x8 puzzle
/// match create_random_puzzle(8, 8) {
///     Ok(field) => // do something with the field
///     Err(err) => // do something with the error
/// }
/// ```
pub fn create_random_puzzle(height: usize, width: usize) -> Result<Field, String> {
    let mut trees = (height * width) / 5;
    // computation for a field of bigger size is very expansive, so using less trees will speed up the process
    if height * width >= TOO_LARGE_FIELD {
        trees = height * width / 6;
    }

    /// Funtion to create an empty `field`.
    /// 
    /// # Arguments
    /// 
    /// * `height` - Height of the puzzle.
    /// * `width` - Width of the puzzle.
    fn create_empty_field(height: usize, width: usize) -> Field {
        Field {
            cells: vec![vec![CellType::Meadow; width]; height],
            row_counts: vec![0; height],
            column_counts: vec![0; width],
            width: width,
            height: height,
            tent_tree_assgs: None
        }
    }

    /// Funtion to reset a given field to be only Celltype::Meadow again.
    /// 
    /// # Arguments
    /// 
    /// * `field` - The field that has to be reset.
    fn reset_field(field: &mut Field) {
        for x in 0..field.cells.len() {
            for y in 0..field.cells[0].len() {
                field.cells[x][y] = CellType::Meadow;
            }
        }
    }

    #[allow(dead_code)]
    fn print_field(field: &Field) {
        for (y, row) in field.cells.iter().enumerate() {
            for cell in row {
                if cell == &CellType::Tent {
                    print!("x");
                } else
                if cell == &CellType::Tree {
                    print!("T");
                }
                else {
                    print!(".");
                }
            }
            println!(" {}",  field.row_counts[y]);
        }
        for c in &field.column_counts {
            print!("{} ", c);
        }
        println!(" ");
    }

    /// Method for placing the tents in an empty field.
    /// Retuns true if successful and false if it couldnt set all tents.
    /// 
    /// # Arguments
    /// 
    /// * `tree_count` - The number of trees to be set.
    /// * `field` - The field in with the trees are going to be placed.
    fn place_tents(tree_count: usize, field: &mut Field) -> bool {
        let mut rng: ThreadRng = rand::thread_rng();
        let height = field.cells.len();
        let width = field.cells[0].len();

        let mut curr_tree_count = tree_count;
        let mut loop_count = 0;
        let mut max_retries = 10000;
        let mut fields_count = field.cells[0].len() * field.cells.len() * 20;

        while curr_tree_count > 0 {
            if loop_count >= fields_count {
                curr_tree_count = tree_count;
                fields_count = 0;
                reset_field(field);
                max_retries -= 1;
            }
            if max_retries <= 0 {
                return false;
            }
            let tree_pos: usize = rng.gen_range(0, height * width - 1);
            let col: usize = tree_pos % width;
            let row: usize = tree_pos / width;
            if field.cells[row][col] != CellType::Tent && !has_tent_neighbours(row, col, &field) {
                field.cells[row][col] = CellType::Tent;
                curr_tree_count -= 1;
            }
            loop_count += 1;
        }
        return true;
    }

    /// Function to get all the neighbour coordinates of a field.
    /// All surrounding fields next to the given fields are neighbours.
    /// Returns a vector of tuples with the cooridnates of the nightbours of a given field. 
    /// 
    /// # Arguments
    /// 
    /// * `col` - The colum of the checked field.
    /// * `row` - The row of the checked field.
    /// * `field` - The field in which the coordinates have to be checked.
    /// * `checked_datatype` - Check for either tents or trees.
    fn get_neighbour_coords(col: usize, row: usize, field: &Field, checked_datatype: CellType) -> Vec<(usize, usize)> {
        let mut coords: Vec<(usize, usize)> = Vec::new();
        let height = field.cells.len();
        let width = field.cells[0].len();

        let mut for_tent_check = false;
        if checked_datatype == CellType::Tent {
            for_tent_check = true;
        }
        let col: isize = col as isize;
        let row: isize = row as isize;

        if col - 1 >= 0 {
            coords.push((row as usize, (col - 1) as usize));
            if row - 1 >= 0 && for_tent_check {
                coords.push(((row - 1) as usize, (col - 1) as usize));
            }
            if row + 1 < height as isize && for_tent_check {
                coords.push(((row + 1) as usize, (col - 1) as usize));
            }
        }
        if col + 1 < width as isize {
            coords.push((row as usize, (col + 1) as usize));
            if row - 1 >= 0 && for_tent_check {
                coords.push(((row - 1) as usize, (col + 1) as usize));
            }
            if row + 1 < height as isize && for_tent_check {
                coords.push(((row + 1) as usize, (col + 1) as usize));
            }
        }
        if row + 1 < height as isize {
            coords.push(((row + 1) as usize, col as usize));
        }
        if row - 1 >= 0 {
            coords.push(((row - 1) as usize, col as usize));
        }
        coords
    }

    /// Function for checking if the checked field has a tent next to it.
    /// Returns true if it has a neighbour, false if there are none.
    /// 
    /// # Arguments
    /// 
    /// * `row` - The row of the checked field.
    /// * `col` - The col of the checked field.
    /// * `field` - The field in which the function checks the coordinates.
    fn has_tent_neighbours(row: usize, col: usize, field: &Field) -> bool {
        let coords: Vec<(usize, usize)> = get_neighbour_coords(col, row, field, CellType::Tent);
        let mut has_neighbour = false;

        if field.cells[row][col] == CellType::Tent {
            return false;
        }
        
        for c in coords {
            if field.cells[c.0][c.1] == CellType::Tent {
                has_neighbour = true;
                break;
            }
        }
        has_neighbour
    }

    /// Function that sets a tree at the given coordinates.
    /// Returns true if successful, false if the tree cannot be set.
    /// 
    /// # Arguments 
    /// 
    /// * `row` - The row of the checked field.
    /// * `col` - The col of the checked field.
    /// * `field` - The field in which the function checks the coordinates.
    fn set_a_tree(row: usize, col: usize, field: &mut Field) -> bool {
        let coords = get_neighbour_coords(col, row, field, CellType::Tree);
        
        let mut can_set = false;
        for c in &coords {
            if field.cells[c.0][c.1] == CellType::Meadow {
                field.cells[c.0][c.1] = CellType::Tree;
                can_set = true;
                break;
            }
        }
        can_set
    }

    /// Function to place trees next to tents. Returns true on success,
    /// false if unsuccessful
    /// 
    /// # Arguments 
    /// 
    /// * `field` - The field in which the trees are placed in.
    fn place_trees(mut field: &mut Field) -> bool {
        let mut is_possible = true;
        
        for y in 0..field.cells.len() {
            for x in 0..field.cells[0].len() {
                if field.cells[y][x] == CellType::Tent {
                    if !set_a_tree(y, x, &mut field) {
                        is_possible = false;
                    } 
                }
            }
        }

        is_possible
    }

    /// Function to fill the side row and column for the amount of tents that are in it.
    /// 
    /// # Arguments 
    /// 
    /// * `field` - The field in which the trees and tents are already placed.
    fn fill_col_row_count(field: &mut Field) {
        for (y, row) in field.cells.iter().enumerate() {
            let mut row_count = 0;
            for cell in row.iter() {
                if cell == &CellType::Tent {
                    row_count += 1;
                }
           }
            field.row_counts[y] = row_count;
        }

        for x1 in 0..field.cells[0].len() {
            let mut col_count = 0;
            for y1 in 0..field.cells.len() {
                if field.cells[y1][x1] == CellType::Tent {
                    col_count += 1;
                }
            }
            field.column_counts[x1] = col_count;
        }
    }

    /// Function to remove all tents from a field
    /// 
    /// # Arguments 
    /// 
    /// * `field` - The field in which the trees and tents are already placed.
    fn remove_tents(field: &mut Field) {
        for y in 0..field.cells.len() {
            for x in 0..field.cells[0].len() {
                if field.cells[y][x] == CellType::Tent {
                    field.cells[y][x] = CellType::Meadow;
                }
            }
        }
    }

    let mut can_create = false;
    let mut field: Field = create_empty_field(height, width);
    let mut loop_count = 0;

    while can_create == false {
        if loop_count >= 100000 {
            return Err("couldnt find a puzzle in 10000 iterations".to_string());
        }
        field = create_empty_field(height, width);
        let tents_worked = place_tents(trees, &mut field);
        if tents_worked {
            can_create = place_trees(&mut field);
        }
        loop_count += 1;
    }
    fill_col_row_count(&mut field);
    remove_tents(&mut field);
    //print_field(&field);
    Ok(field)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc5x5() {
        match create_random_puzzle(5, 5) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn calc10x10() {
        match create_random_puzzle(10, 10) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn calc15x15() {
        match create_random_puzzle(15, 15) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn calc20x20() {
        match create_random_puzzle(20, 20) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn calc25x25() {
        match create_random_puzzle(25, 25) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn calc30x30() {
        match create_random_puzzle(30, 30) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }


    #[test]
    fn calc10x5() {
        match create_random_puzzle(10, 5) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn calc15x5() {
        match create_random_puzzle(15, 5) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn calc5x10() {
        match create_random_puzzle(5, 10) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn calc5x15() {
        match create_random_puzzle(5, 15) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }
}
