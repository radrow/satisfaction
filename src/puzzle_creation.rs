use crate::field::*;
use rand::prelude::*;

pub fn create_random_puzzle(hight: usize, width: usize, trees: usize) -> Field {
    fn create_empty_field(hight: usize, width: usize) -> Field {
        Field {
            cells: vec![vec![CellType::Meadow; width]; hight],
            row_counts: vec![0; hight],
            column_counts: vec![0; width]
        }
    }

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

    fn place_trees(mut tree_count: usize, field: &mut Field) {
        let mut rng: ThreadRng = rand::thread_rng();
        let hight = field.cells.len();
        let width = field.cells[0].len();

        while tree_count > 0 {
            let tree_pos: usize = rng.gen_range(0, hight * width - 1);
            let col: usize = tree_pos % width;
            let row: usize = tree_pos / width;
            if field.cells[row][col] != CellType::Tree {
                field.cells[row][col] = CellType::Tree;
                tree_count -= 1;
            }
        }
    }

    fn get_neighbour_coords(col: usize, row: usize, field: &Field, checked_datatype: CellType) -> Vec<(usize, usize)> {
        let mut coords: Vec<(usize, usize)> = Vec::new();
        let hight = field.cells.len();
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
            if row + 1 < hight as isize && for_tent_check {
                coords.push(((row + 1) as usize, (col - 1) as usize));
            }
        }
        if col + 1 < width as isize {
            coords.push((row as usize, (col + 1) as usize));
            if row - 1 >= 0 && for_tent_check {
                coords.push(((row - 1) as usize, (col + 1) as usize));
            }
            if row + 1 < hight as isize && for_tent_check {
                coords.push(((row + 1) as usize, (col + 1) as usize));
            }
        }
        if row + 1 < hight as isize {
            coords.push(((row + 1) as usize, col as usize));
        }
        if row - 1 >= 0 {
            coords.push(((row - 1) as usize, col as usize));
        }
        coords
    }

    fn has_tent_neighbours(row: usize, col: usize, field: &mut Field) -> bool {
        let coords: Vec<(usize, usize)> = get_neighbour_coords(col, row, field, CellType::Tent);
        let mut has_nighbour = false;

        if field.cells[row][col] == CellType::Tent {
            return false;
        }
        
        for c in coords {
            if field.cells[c.0][c.1] == CellType::Tent {
                has_nighbour = true;
                break;
            }
        }
        has_nighbour
    }

    fn set_a_tent(row: usize, col: usize, field: &mut Field) -> bool {
        let coords = get_neighbour_coords(col, row, field, CellType::Tree);
        
        let mut can_set = false;
        for c in &coords {
            if !has_tent_neighbours(c.0, c.1, field) {
                if field.cells[c.0][c.1] == CellType::Meadow {
                    field.cells[c.0][c.1] = CellType::Tent;
                    can_set = true;
                    break;
                }
            }
        }
        can_set
    }

    fn create_possible_tent_pos(mut field: &mut Field) -> bool {
        let mut is_possible = true;
        
        for y in 0..field.cells.len() {
            for x in 0..field.cells[0].len() {
                if field.cells[y][x] == CellType::Tree {
                    if !set_a_tent(y, x, &mut field) {
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
            let mut col_count = 0;
            for (x, cell) in row.iter().enumerate() {
                if cell == &CellType::Tent {
                    row_count += 1;
                }
                if field.cells[x][y] == CellType::Tent {
                    col_count += 1;
                }
            }
            field.column_counts[y] = col_count;
            field.row_counts[y] = row_count;
        }
    }

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
    let mut field: Field = create_empty_field(hight, width);

    while can_create == false {
        field = create_empty_field(hight, width);
        place_trees(trees, &mut field);
        can_create = create_possible_tent_pos(&mut field);
    }
    fill_col_row_count(&mut field);
    remove_tents(&mut field);
    field
}