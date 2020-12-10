use std::{collections::{HashMap}};
use std::io;
use std::path::Path;
use std::fs;
use std::process::{Command, Stdio};
use std::io::Write;

use itertools::Itertools;

type TentPlace = (usize, usize);
type TreePlace = (usize, usize);

type AxisSet = HashMap<usize, Vec<usize>>;
type NeiSet = Vec<(usize, usize)>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CellType {
    Tent,
    Tree,
    Meadow,
}

pub struct Field {
    pub cells: Vec<Vec<CellType>>,
    pub row_counts: Vec<usize>,
    pub column_counts: Vec<usize>,
    pub height : usize,
    pub width : usize,
}

#[derive (PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
struct Assignment {
    tent : TentPlace,
    tree : TreePlace
}

impl Field {
    #[allow(dead_code)]
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

    pub fn tent_coordinates(&self) -> Vec<Vec<TentPlace>> {
        let width = self.cells.len();
        let height = self.cells[0].len();
        let mut tents_by_trees = Vec::new();

        for (x, row) in self.cells.iter().enumerate() {
            for (y, cell) in row.iter().enumerate() {
                if *cell == CellType::Tree {
                    let left = x as isize - 1;
                    let right = x + 1;
                    let top = y as isize - 1;
                    let bottom = y + 1;
                    let mut potential_tents = Vec::with_capacity(4);

                    if left >= 0 && self.cells[left as usize][y] != CellType::Tree {
                        potential_tents.push((left as usize, y));
                    }
                    if right < width && self.cells[right][y] != CellType::Tree {
                        potential_tents.push((right, y));
                    }
                    if top >= 0 && self.cells[x][top as usize] != CellType::Tree {
                        potential_tents.push((x, top as usize));
                    }
                    if bottom < height && self.cells[x][bottom] != CellType::Tree {
                        potential_tents.push((x, bottom));
                    }
                    tents_by_trees.push(potential_tents);
                }
            }
        }
        tents_by_trees
    }

    fn to_formula(&self) -> (String, HashMap<usize,TentPlace>, Vec<(Assignment, usize)>) {
        let tents = self.tent_coordinates();

        // Id to coordinate
        let tent_mapping = tents.iter()
            .flatten()
            .unique()
            .enumerate()
            .map(|(id, coord)| (id+1, *coord))
            .collect::<HashMap<usize,TentPlace>>();

        // Coordinate to id
        let id_mapping = tent_mapping.iter()
            .map(|(id, coord)| (*coord, *id))
            .collect::<HashMap<TentPlace, usize>>();

        let col_set: AxisSet = Field::make_coord_set_by(
            &|x : &TentPlace| -> usize {x.1}, &tent_mapping
        );
        let row_set: AxisSet = Field::make_coord_set_by(
            &|x : &TentPlace| -> usize {x.0}, &tent_mapping
        );
        let neighbour_set: NeiSet = Field::make_neighbour_set(&tent_mapping, &id_mapping);

        let mut total = "p cnf 1 1\n".to_string();
        total.push_str(&Field::make_count_constraints(&self.column_counts, &col_set));
        total.push_str(&Field::make_count_constraints(&self.row_counts, &row_set));
        total.push_str(&Field::make_neighbour_constraints(&neighbour_set));
        let (neistr, assg_mapping) =
            &Field::make_correspondence_constraints(&self, &id_mapping);
        total.push_str(neistr);
        (total, tent_mapping, assg_mapping.clone().to_vec())
    }

    pub fn solve(&mut self) -> Vec<((usize, usize), (usize, usize))> {
        let (formular, t_mapping, a_mapping) = self.to_formula();
        let mut process = Command::new("cadical")
            .arg("-f")
            .arg("-q")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let stdin = process.stdin.as_mut().unwrap();
        stdin.write_all(formular.as_bytes()).unwrap();
        let output = process.wait_with_output().unwrap();
        let output_string = String::from_utf8(output.stdout).unwrap();

        let vec: Vec<TentPlace> = output_string.lines()
            .skip(1)
            .map(|line| {
                line.split_ascii_whitespace()
                    .into_iter()
                    .skip(1)
                    .map(|number| {
                        number.parse::<isize>().unwrap()
                    })
                    .filter(
                        |number| { *number > 0 && (*number as usize) <= t_mapping.len()}
                    ).map(|id| {
                        *t_mapping.get(&(id as usize)).unwrap()
                    })
            }).flatten()
                .collect();

        for (x,y) in vec.into_iter() {
            self.cells[x][y] = CellType::Tent;
        }

        a_mapping
            .iter()
            .filter(|(_, i)| *i > 0)
            .map(|(a, _)| (a.tent, a.tree))
            .collect()
    }


    fn make_coord_set_by (
        by : &dyn Fn(&TentPlace) -> usize,
        tents : &HashMap<usize, TentPlace>,
    ) -> AxisSet {
        let mut out : AxisSet = HashMap::new();

        for (id, coord) in tents {
            out.entry(by(coord))
                .and_modify(|v| v.push(*id))
                .or_insert(vec![*id]);
        }
        out
    }

    fn make_neighbour_set(id_to_coord : &HashMap<usize, TentPlace>, coord_to_id: &HashMap<TentPlace, usize>) -> NeiSet {
        let mut out : NeiSet = Vec::new();

        for (id, tent) in id_to_coord {
            let mut neighbours = vec![];

            let mut candidates = vec![];
            candidates.push((tent.0 + 1, tent.1));
            candidates.push((tent.0, tent.1 + 1));
            if tent.1 > 0 {
                candidates.push((tent.0 + 1, tent.1 - 1))
            }
            candidates.push((tent.0 + 1, tent.1 + 1));

            for candidate in candidates {

                if coord_to_id.contains_key(&candidate) {
                    neighbours.push((*id, coord_to_id[&candidate]));
                }
            }

            out.extend(neighbours.into_iter());
        }

        out
    }

    fn make_count_constraints(counts : &Vec<usize>, axes : &AxisSet) -> String {
        let mut clauses = String::new();

        // Conjunction of constranits per each axis
        for (axis, tents) in axes {
            let axis_count = counts[*axis];
            if axis_count == 0 {
                continue;
            }
            let for_axis = Field::axis_constraint(tents, axis_count);
            clauses.push_str(&for_axis);
        }
        clauses
    }


    fn make_neighbour_constraints(neigh : &NeiSet) -> String {
        let mut out = neigh.iter()
            .map(|(x,y)|{
                format!("-{} -{}", x, y) // Not both neighbours
            }).join(" 0\n");
        out.push_str(" 0\n");
        out
    }

    fn make_correspondence_constraints(
        &self,
        id_mapping: &HashMap<TentPlace, usize>) -> (String, Vec<(Assignment, usize)>) {

        let mut oneof_packs : Vec<Vec<Assignment>> = vec![];
        let mut cond_oneof_packs : Vec<(TentPlace, Vec<Assignment>)> = vec![];

        let mut assignments : Vec<Assignment> = vec![];
        for (x, row) in self.cells.iter().enumerate() {
            for (y, cell) in row.iter().enumerate() {
                if *cell == CellType::Tree || *cell == CellType::Meadow {
                    let is_tree = *cell == CellType::Tree;
                    let looking_for = if is_tree {CellType::Meadow} else {CellType::Tree};

                    let left = x as isize - 1;
                    let right = x + 1;
                    let top = y as isize - 1;
                    let bottom = y + 1;

                    let mut pack : Vec<Assignment> = vec![];

                    if left >= 0 && self.cells[left as usize][y] == looking_for {
                        let asg = if is_tree {
                            Assignment{
                                tent: (left as usize, y),
                                tree: (x, y)
                            }
                        } else {
                            Assignment{
                                tree: (left as usize, y),
                                tent: (x, y)
                            }
                        };
                        pack.push(asg);
                        assignments.push(asg);
                    }
                    if right < self.width && self.cells[right][y] == looking_for {
                        let asg = if  is_tree {
                            Assignment{
                                tent: (right as usize, y),
                                tree: (x, y)
                            }
                        } else {
                            Assignment{
                                tree: (right as usize, y),
                                tent: (x, y)
                            }
                        };
                        pack.push(asg);
                        assignments.push(asg);
                    }
                    if top >= 0 && self.cells[x][top as usize] == looking_for {
                        let asg = if  is_tree {
                            Assignment{
                                tent: (x, top as usize),
                                tree: (x, y)
                            }
                        } else {
                            Assignment{
                                tree: (x, top as usize),
                                tent: (x, y)
                            }
                        };
                        pack.push(asg);
                        assignments.push(asg);
                    }
                    if bottom < self.height && self.cells[x][bottom] == looking_for {
                        let asg = if  is_tree {
                            Assignment{
                                tent: (x, bottom as usize),
                                tree: (x, y)
                            }
                        } else {
                            Assignment{
                                tree: (x, bottom as usize),
                                tent: (x, y)
                            }
                        };
                        pack.push(asg);
                        assignments.push(asg);
                    }
                    if pack.len() != 0 {
                        if is_tree {
                            oneof_packs.push(pack);
                        } else {
                            cond_oneof_packs.push(((x, y), pack));
                        }
                    }
                }
            }
        }

        assignments.sort();
        assignments.dedup();

        let assg_mapping = assignments.iter()
            .unique()
            .enumerate()
            .map(|(id, coord)| (id + id_mapping.len() + 1, *coord))
            .collect::<HashMap<usize, Assignment>>();

        let assg_id_mapping = assg_mapping.iter()
            .map(|(id, coord)| (*coord, *id))
            .collect::<HashMap<Assignment, usize>>();


        fn makeoneof(
            id_mapping : &HashMap<TentPlace, usize>,
            assg_id_mapping : &HashMap<Assignment, usize>,
            pack : &Vec<Assignment>,
            cond : Option<TentPlace>
        ) -> String {
            let ids : Vec<&usize> = pack
                .iter()
                .map(|assg| assg_id_mapping
                     .get(assg)
                     .unwrap()
                     ).collect();
            let any_of =
                match cond {
                    Some(x) => String::from("-") + &id_mapping.get(&x).unwrap().to_string()
                        + " " + &ids.iter().map(|x| x.to_string()).join(" "),
                    None => ids.iter().map(|x| x.to_string()).join(" ")
                };

            let no_two = {
                let mut out = vec![];
                for i in &ids {
                    for j in &ids {
                        if i < j {
                            out.push(
                                match cond {
                                    Some(x) => String::from("-") + &id_mapping.get(&x).unwrap().to_string()
                                        + " -" + &i.to_string() + " -" + &j.to_string(),
                                    None => String::from("-") + &i.to_string() + " -" + &j.to_string()
                                });
                        }
                    }
                }
                out.join(" 0 \n")
            };
            if pack.len() == 1 {
                any_of
            }
            else {
                vec![any_of, no_two].join(" 0 \n")
            }
        }

        let pack_formula =
            oneof_packs
            .iter()
            .map(|p| makeoneof(&id_mapping, &assg_id_mapping, p, None))
            .join(" 0 \n");

        let cond_pack_formula =
            cond_oneof_packs
            .iter()
            .map(|(cond, p)| makeoneof(&id_mapping, &assg_id_mapping, p, Some(*cond)))
            .join(" 0 \n");

        let tent_exists_formula =
            assignments
            .iter()
            .map(|a| id_mapping.get(&a.tent).unwrap().to_string() + " -"
                 + &assg_id_mapping.get(a).unwrap().to_string())
            .join(" 0 \n");

        (vec![pack_formula,
             cond_pack_formula,
             tent_exists_formula
        ].iter().join(" 0 \n") + " 0",
         assg_id_mapping
         .iter().map(|(a, i)| (*a, *i))
         .collect::<Vec<(Assignment, usize)>>()
        )
    }

    pub fn axis_constraint(variables: &Vec<usize>, count: usize) -> String {
        let mut lower_bound_clauses = variables.iter()
            .map(|v| v.to_string())
            .combinations(variables.len()-count+1)
            .map(|v| {
                v.join(" ")
            }).join(" 0 \n");
        lower_bound_clauses.push_str("  0 \n");

        let mut upper_bound_clauses = variables.iter()
            .map(|v| format!("-{}",*v))
            .combinations(count+1)
            .map(|v| {
                v.join(" ")
            }).join(" 0 \n");
        upper_bound_clauses.push_str(
            if !upper_bound_clauses.is_empty() { " 0 \n" }
            else { "\n" }
        );

        lower_bound_clauses.push_str(&upper_bound_clauses);
        lower_bound_clauses
    }
}
