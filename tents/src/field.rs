use std::{collections::{HashMap}};
use std::io;
use std::path::Path;
use std::fs;

use itertools::Itertools;
use rayon::prelude::*;
use rayon::iter::*;
use rayon;

use crate::cnf::{CNF, CNFClause, CNFVar};

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

    fn to_formula(&self) -> (CNF, HashMap<TentPlace, usize>, Vec<(Assignment, usize)>) {
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

        let mut total = CNF::new();
        let ((count_col_form, count_row_form), (neigs_form, (corr_form, assg_mapping))) =
            rayon::join(
                | | rayon::join(| | Field::make_count_constraints(&self.column_counts, &col_set),
                                | | Field::make_count_constraints(&self.row_counts, &row_set)
                ),
                | | rayon::join(| | Field::make_neighbour_constraints(&neighbour_set),
                                | | Field::make_correspondence_constraints(&self, &id_mapping)
                )
            );
        total.extend(count_col_form);
        total.extend(count_row_form);
        total.extend(neigs_form);
        total.extend(corr_form);
        (total, id_mapping, assg_mapping.clone().to_vec())
    }

    pub fn solve(&mut self) -> Vec<((usize, usize), (usize, usize))> {
        let (formula, t_mapping, a_mapping) = self.to_formula();

        let mut solver = formula.to_solver();
        match solver.solve() {
            None => panic!("was zur hölle"),
            Some(satisfiable) => {
                if satisfiable {
                    for y in 0..self.height {
                        for x in 0..self.width {
                            match
                                t_mapping.get(&(y, x))
                                .and_then(|var_name| solver.value(*var_name as i32)) {
                                    None => (),
                                    Some(true) => self.cells[y][x] = CellType::Tent,
                                    Some(false) => self.cells[y][x] = CellType::Meadow
                                }
                        }
                    }

                    a_mapping
                        .iter()
                        .filter(|(_, i)| *i > 0)
                        .map(|(a, _)| (a.tent, a.tree))
                        .collect()
                }
                else {
                    println!("No solution");
                    vec![]
                }
            }
        }
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

    fn make_count_constraints(counts : &Vec<usize>, axes : &AxisSet) -> CNF {
        let mut clauses = CNF::new();

        // Conjunction of constranits per each axis
        for (axis, tents) in axes {
            let axis_count = counts[*axis];
            if axis_count == 0 {
                continue;
            }
            let for_axis = Field::axis_constraint(tents, axis_count);
            clauses.extend(for_axis);
        }
        clauses
    }


    fn make_neighbour_constraints(neigh : &NeiSet) -> CNF {
        neigh
            .iter()
            .map(|(x,y)|{
                CNFClause{vars: vec![CNFVar::Neg(*x as u32), CNFVar::Neg(*y as u32)]}
            }).collect()
    }

    fn make_correspondence_constraints(
        &self,
        id_mapping: &HashMap<TentPlace, usize>) -> (CNF, Vec<(Assignment, usize)>) {

        let mut oneof_packs : Vec<Vec<Assignment>> = vec![];
        let mut cond_oneof_packs : Vec<(TentPlace, Vec<Assignment>)> = vec![];

        let width = self.cells.len();
        let height = self.cells[0].len();
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
                    if right < width && self.cells[right][y] == looking_for {
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
                    if bottom < height && self.cells[x][bottom] == looking_for {
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
        ) -> Vec<CNFClause> {
            let ids : Vec<&usize> = pack
                .iter()
                .map(|assg| assg_id_mapping
                     .get(assg)
                     .unwrap()
                     ).collect();
            let mut any_of =
                vec![
                    match cond {
                        Some(x) => {
                            let mut c = ids.iter().map(|i| CNFVar::Pos(**i as u32)).collect::<CNFClause>();
                            c.push(CNFVar::Neg(*id_mapping.get(&x).unwrap() as u32));
                            c
                        },
                        None => ids.iter().map(|i| CNFVar::Pos(**i as u32)).collect::<CNFClause>()
                    }
                ];

            let no_two = {
                let mut out = vec![];
                for i in &ids {
                    for j in &ids {
                        if i < j {
                            out.push(
                                match cond {
                                    Some(x) => {
                                        let mut c = CNFClause::new();
                                        c.push(CNFVar::Neg(**i as u32));
                                        c.push(CNFVar::Neg(**j as u32));
                                        c.push(CNFVar::Neg(*id_mapping.get(&x).unwrap() as u32));
                                        c
                                    },
                                    None => {
                                        let mut c = CNFClause::new();
                                        c.push(CNFVar::Neg(**i as u32));
                                        c.push(CNFVar::Neg(**j as u32));
                                        c
                                    }
                                });
                        }
                    }
                }
                out
            };
            any_of.extend(no_two);
            any_of
        }

        let pack_formula =
            oneof_packs
            .par_iter()
            .flat_map(|p| makeoneof(&id_mapping, &assg_id_mapping, p, None));

        let cond_pack_formula =
            cond_oneof_packs
            .par_iter()
            .flat_map(|(cond, p)| makeoneof(&id_mapping, &assg_id_mapping, p, Some(*cond)));

        let tent_exists_formula =
            assignments
            .par_iter()
            .map(|a|
                 CNFClause{
                     vars: vec![
                         CNFVar::Neg(*assg_id_mapping.get(a).unwrap() as u32),
                         CNFVar::Pos(*id_mapping.get(&a.tent).unwrap() as u32)
                     ]
                 });

        (pack_formula.chain(cond_pack_formula).chain(tent_exists_formula).collect(),
         assg_id_mapping
         .iter().map(|(a, i)| (*a, *i))
         .collect::<Vec<(Assignment, usize)>>()
        )
    }

    pub fn axis_constraint(variables: &Vec<usize>, count: usize) -> CNF {
        let lower_bound_clauses =
            variables.iter()
            .map(|v| *v as u32)
            .combinations(variables.len() - count + 1).par_bridge()
            .map(|vs| vs.iter().map(|v| CNFVar::Pos(*v)).collect::<CNFClause>());

        let upper_bound_clauses =
            variables.iter()
            .map(|v| *v as u32)
            .combinations(count+1).par_bridge()
            .map(|vs| vs.iter().map(|v| CNFVar::Neg(*v)).collect::<CNFClause>());

        lower_bound_clauses.chain(upper_bound_clauses)
            .collect::<CNF>()
    }
}
