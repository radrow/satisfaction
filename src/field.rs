use std::collections::{HashSet, HashMap};
use std::io;
use std::path::Path;
use std::fs;

use cadical;

use crate::formula::*;


type TentPlace = (usize, usize);

type AxisSet = HashMap<usize, Vec<TentPlace>>;
type NeiSet = HashMap<TentPlace, Vec<TentPlace>>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CellType {
    Unknown,
    Tent,
    Tree,
    Meadow,
}


pub fn mk_var_name((x, y) : TentPlace) -> String {
    return String::from("v_") + &x.to_string() + "_" + &y.to_string();
}

pub struct Field {
    pub cells: Vec<Vec<CellType>>,
    pub row_counts: Vec<usize>,
    pub column_counts: Vec<usize>,
}

impl Field {
    pub fn from_file(path: &Path) -> io::Result<Field> {
        let contents: String = fs::read_to_string(path)?;
        let mut split = contents.split(|c| c == '\n' || c == ' ' || c == '\r');

        let height: usize = split.next().unwrap().parse::<usize>().unwrap();
        let width: usize = split.next().unwrap().parse::<usize>().unwrap();

        let mut field: Vec<String> = Vec::new();
        let mut row_counts: Vec<usize> = Vec::new();
        println!("height: {}", height);
        for _ in 0..height {
            field.push(split.next().unwrap().to_string());
            row_counts.push(split.next().unwrap().parse::<usize>().unwrap());
        }

        let column_counts: Vec<usize> = split.filter_map(
            |x| match x.parse::<usize>() {
                Err(_) => None,
                Ok(x) => Some(x)
            }
        ).collect();

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

    pub fn tent_coordinates(&self) -> HashSet<TentPlace> {
        let mut tent_coordinates: HashSet<TentPlace> = HashSet::new();
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

    pub fn to_dimacs(&self) -> String {
        self.to_formula().to_cnf().to_dimacs()
    }

    pub fn to_solver(&self) -> cadical::Solver {
        self.to_formula().to_cnf().to_solver()
    }

    pub fn to_formula(&self) -> Formula {
        let tents = &self.tent_coordinates();

        let col_set: AxisSet = Field::make_coord_set_by(
            &|x : &TentPlace| -> usize {x.1}, tents
        );
        let row_set: AxisSet = Field::make_coord_set_by(
            &|x : &TentPlace| -> usize {x.0}, tents
        );
        let nei_set: NeiSet = Field::make_nei_set(tents);

        let col_constraints : Formula =
            Field::make_count_constraints(&self.column_counts, &col_set);
        let row_constraints : Formula =
            Field::make_count_constraints(&self.row_counts, &row_set);
        let nei_constraints : Formula =
            Field::make_nei_constraints(&nei_set);

        col_constraints.and(row_constraints).and(nei_constraints)
    }


    fn make_coord_set_by (
        by : &dyn Fn(&TentPlace) -> usize,
        tents : &HashSet<TentPlace>,
    ) -> AxisSet {
        let mut out : AxisSet = HashMap::new();

        for tent in tents {
            out.entry(by(tent))
                .and_modify(|v| v.push(*tent))
                .or_insert(vec![*tent]);
        }

        out
    }

    fn make_nei_set(tents : &HashSet<TentPlace>) -> NeiSet {
        let mut out : NeiSet = HashMap::new();

        for tent in tents {
            let mut neighbours = vec![];

            let mut candidates = vec![];
            if tent.0 > 0 {
                candidates.push((tent.0 - 1, tent.1))
            }
            if tent.1 > 0 {
                candidates.push((tent.0, tent.1 - 1))
            }
            candidates.push((tent.0 + 1, tent.1));
            candidates.push((tent.0, tent.1 + 1));

            if tent.0 > 0 && tent.1 > 0 {
                candidates.push((tent.0 - 1, tent.1 - 1))
            }
            if tent.0 > 0 {
                candidates.push((tent.0 - 1, tent.1 + 1))
            }
            if tent.1 > 0 {
                candidates.push((tent.0 + 1, tent.1 - 1))
            }
            candidates.push((tent.0 + 1, tent.1 + 1));

            for candidate in candidates {
                if tents.contains(&candidate) {
                    neighbours.push(candidate)
                }
            }

            out.insert(*tent, neighbours);
        }

        out
    }

    fn make_count_constraints(counts : &Vec<usize>, axes : &AxisSet) -> Formula {
        // The default option is true – if we have no constraints
        let mut clauses = Formula::Const(true);

        // Conjunction of constranits per each axis
        for (axis, tents) in axes {
            let axis_count = counts[*axis];
            clauses = clauses.and(Field::make_count_constraints_for_axis(axis_count, tents))
        }
        clauses
    }

    fn make_count_constraints_for_axis(axis_count : usize, tents : &Vec<TentPlace>) -> Formula {
        fn go(count : usize, index : usize, tents : &Vec<TentPlace>) -> Formula {
            if index >= tents.len() {
                Formula::Const(true)
            } else if count == 0 {
                Formula::Var(mk_var_name(tents[index])).not().and(go(count, index + 1, tents))
            } else {
                let var1 = Formula::Var(mk_var_name(tents[index]));
                let var2 = Formula::Var(mk_var_name(tents[index]));
                (var1.and(go(count - 1, index + 1, tents)))
                    .or(var2.not().and(go(count, index + 1, tents)))
            }
        }
        go(axis_count, 0, tents)
    }

    fn make_nei_constraints(neigh : &NeiSet) -> Formula {
        let mut out = Formula::Const(true);
        for (t, ns) in neigh {
            for n in ns {
                out = out.and(Formula::Var(mk_var_name(*t)).not().iff(Formula::Var(mk_var_name(*n))))
            }
        }
        out
    }

}
