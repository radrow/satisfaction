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
    pub height : usize,
    pub width : usize,
}

impl Field {
    pub fn create_empty(width: usize, height: usize) -> Field {
        let prototype = vec![CellType::Meadow;width];
        let cells = vec![prototype; height];

        let column_counts = vec![0, width];
        let row_counts = vec![0; height];

        Field {
            cells,
            height: row_counts.len(),
            width: column_counts.len(),
            row_counts,
            column_counts,
        }
    }

    pub fn from_file(path: &Path) -> io::Result<Field> {
        let contents: String = fs::read_to_string(path)?;
        let mut split = contents.split(|c| c == '\n' || c == ' ' || c == '\r');

        let height: usize = split.next().unwrap().parse::<usize>().unwrap();
        let width: usize = split.next().unwrap().parse::<usize>().unwrap();

        let mut field: Vec<String> = Vec::new();
        let mut row_counts: Vec<usize> = Vec::new();


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
            cells,
            height: height,
            width: width
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

        println!("\nCOLS:");
        let col_constraints : Formula =
            Field::make_count_constraints(&self.column_counts, &col_set);
        println!("\nROWS:");
        let row_constraints : Formula =
            Field::make_count_constraints(&self.row_counts, &row_set);
        let nei_constraints : Formula =
            Field::make_nei_constraints(&nei_set);

        println!("\nNEIS: {} \n", nei_constraints);

        col_constraints.and(row_constraints).and(nei_constraints)
    }

    pub fn solve(&mut self) {
        let formula = self.to_formula();
        let cnf = formula.to_cnf();

        println!("\nCNF: {}\n", cnf);
        let var_map = cnf.create_variable_mapping();
        let mut solver = cnf.to_solver();
        match solver.solve() {
            None => panic!(":(((("),
            Some(satisfiable) => {
                solver.write_dimacs(Path::new("/tmp/xd"));
                if satisfiable {
                    for y in 0..self.height {
                        for x in 0..self.width {
                            match
                                var_map.get(mk_var_name((y, x)).as_str())
                                .and_then(|var_name| solver.value(*var_name)) {
                                    None => (),
                                    Some(true) => self.cells[y][x] = {
                                        CellType::Tent
                                    },
                                    Some(false) => self.cells[y][x] = CellType::Meadow
                                }
                        }
                    }
                }
                else {
                    panic!("no solution")
                }
            }
        }
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
            candidates.push((tent.0 + 1, tent.1));
            candidates.push((tent.0, tent.1 + 1));
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
            let for_axis = Field::make_count_constraints_for_axis(axis_count, tents);
            println!("{}: {}", axis, for_axis);
            clauses = clauses.and(for_axis)
        }
        clauses
    }

    fn make_count_constraints_for_axis(axis_count : usize, tents : &Vec<TentPlace>) -> Formula {
        fn go(count : usize, index : usize, tents : &Vec<TentPlace>) -> Formula {
            if index >= tents.len() {
                Formula::Const(count == 0)
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
                out = out.and(
                    Formula::Var(mk_var_name(*t)).not()
                        .or(Formula::Var(mk_var_name(*n)).not()))
            }
        }
        out
    }
}
