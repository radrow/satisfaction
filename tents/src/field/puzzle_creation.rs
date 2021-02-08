use super::field::{Field, CellType};
use rand::prelude::*;

pub fn create_random_puzzle(width: usize, height: usize) -> Result<Field, Box<dyn std::error::Error>> {
    fn reset_field(field: &mut Field) {
        for x in 0..field.cells.len() {
            for y in 0..field.cells[0].len() {
                field.cells[x][y] = CellType::Meadow;
            }
        }
    }

    fn place_tents(tree_count: usize, field: &mut Field) -> bool {
        let mut rng: ThreadRng = rand::thread_rng();

        let mut curr_tree_count = tree_count;
        let mut loop_count = 0;
        let mut max_retries = 10000;
        let mut fields_count = field.width * field.height * 20;

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
            let tree_pos: usize = rng.gen_range(0, field.height * field.width - 1);
            let col: usize = tree_pos % field.width;
            let row: usize = tree_pos / field.width;
            if field.get_cell(row, col) != CellType::Tent && !has_tent_neighbours(row, col, &field) {
                *field.get_cell_mut(row, col) = CellType::Tent;
                curr_tree_count -= 1;
            }
            loop_count += 1;
        }
        return true;
    }

    fn get_neighbour_coords(row: usize, col: usize, field: &Field, checked_datatype: CellType) -> Vec<(usize, usize)> {
        let mut coords: Vec<(usize, usize)> = Vec::new();

        let mut for_tent_check = false;
        if checked_datatype == CellType::Tent {
            for_tent_check = true;
        }

        if let Some(left) = col.checked_sub(1) {
            coords.push((row, left));
            if let Some(top) = row.checked_sub(1) {
                if for_tent_check {
                    coords.push((top, left));
                }
            }
            let bottom = row + 1;
            if bottom < field.height && for_tent_check {
                coords.push((bottom, left));
            }
        }

        let right = col + 1;
        if right < field.width {
            coords.push((row, right));
            if let Some(top) = row.checked_sub(1) {
                if for_tent_check {
                    coords.push((top, right));
                }
            }
            let bottom = row + 1;
            if bottom < field.height && for_tent_check {
                coords.push((bottom, right));
            }
        }
        let top = row + 1;
        if top < field.height {
            coords.push((top, col));
        }
        if let Some(top) = row.checked_sub(1) {
            coords.push((top, col));
        }
        coords
    }

    fn has_tent_neighbours(row: usize, col: usize, field: &Field) -> bool {
        let coords: Vec<(usize, usize)> = get_neighbour_coords(row, col, field, CellType::Tent);
        let mut has_neighbour = false;

        if field.get_cell(row, col) == CellType::Tent {
            return false;
        }
        
        for (row, col) in coords {
            if field.get_cell(row, col) == CellType::Tent {
                has_neighbour = true;
                break;
            }
        }
        has_neighbour
    }

    fn set_a_tree(row: usize, col: usize, field: &mut Field) -> bool {
        let coords = get_neighbour_coords(row, col, field, CellType::Tree);
        
        let mut can_set = false;
        for (row, col) in &coords {
            let cell = field.get_cell_mut(*row, *col);
            if *cell == CellType::Meadow {
                *cell = CellType::Tree;
                can_set = true;
                break;
            }
        }
        can_set
    }

    fn place_trees(mut field: &mut Field) -> bool {
        let mut is_possible = true;
        
        for row in 0..field.height {
            for column in 0..field.width {
                if field.get_cell(row, column) == CellType::Tent {
                    if !set_a_tree(row, column, &mut field) {
                        is_possible = false;
                    } 
                }
            }
        }

        is_possible
    }

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

        for column in 0..field.width {
            let mut col_count = 0;
            for row in 0..field.height {
                if field.get_cell(row, column) == CellType::Tent {
                    col_count += 1;
                }
            }
            field.column_counts[column] = col_count;
        }
    }

    fn remove_tents(field: &mut Field) {
        for row in 0..field.height {
            for column in 0..field.width {
                if field.get_cell(row, column) == CellType::Tent {
                    *field.get_cell_mut(row, column) = CellType::Meadow;
                }
            }
        }
    }

    let mut can_create = false;
    let mut field = Field::create_empty(width, height)?;
    let trees = (height * width) / 5;
    let mut loop_count = 0;

    while can_create == false {
        if loop_count >= 100000 {
            return Err("couldnt find a puzzle in 10000 iterations".into());
        }
        field = Field::create_empty(width, height)?;
        let tents_worked = place_tents(trees, &mut field);
        if tents_worked {
            can_create = place_trees(&mut field);
        }
        loop_count += 1;
    }
    fill_col_row_count(&mut field);
    remove_tents(&mut field);
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
