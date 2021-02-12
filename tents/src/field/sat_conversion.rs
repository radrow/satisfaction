use itertools::Itertools;
use rayon;
use rayon::prelude::*;
use std::collections::HashMap;

use solver::cnf::{CNFClause, CNFVar, VarId, CNF};
use solver::{solvers::InterruptibleSolver, SATSolution};

use super::field::{CellType, Field, TentPlace};

/// Coordinate of a tent
pub type TreePlace = (usize, usize);

/// Constraints on the number of tents at given axis
pub type AxisSet = HashMap<usize, Vec<usize>>;
/// Constraints on the neighbourhood of tents
pub type NeighbourSet = Vec<(usize, usize)>;

/// Connection between a tree and assigned tent
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct Assignment {
    pub tent: TentPlace,
    pub tree: TreePlace,
}

/// Solve the puzzle
pub async fn field_to_cnf(mut field: Field, solver: &impl InterruptibleSolver) -> Option<Field> {
    let (formula, t_mapping) = to_formula(&field);

    match solver.solve_interruptible(&formula).await {
        SATSolution::Satisfiable(assignment) => {
            for row in 0..field.height {
                for column in 0..field.width {
                    match t_mapping
                        .get(&(row, column))
                        .map(|var_name| assignment[*var_name - 1])
                    {
                        None => (),
                        Some(true) => *field.get_cell_mut(row, column) = CellType::Tent,
                        Some(false) => *field.get_cell_mut(row, column) = CellType::Meadow,
                    }
                }
            }
            Some(field)
        }
        SATSolution::Unsatisfiable | SATSolution::Unknown => None,
    }
}

/// Mapping from tent places to variable ids
pub fn make_id_mapping(field: &Field) -> HashMap<TentPlace, usize> {
    let tents = field.tent_coordinates();
    tents
        .iter()
        .unique()
        .enumerate()
        .map(|(id, coord)| (*coord, id + 1))
        .collect::<HashMap<TentPlace, usize>>()
}
/// Compiles the puzzle to CNF. On the latter positions returns mapings
/// from tent places to assiociated variables and list of tree-tent
/// assignments with assiociated variables.
pub fn to_formula(field: &Field) -> (CNF, HashMap<TentPlace, VarId>) {
    let id_mapping: HashMap<TentPlace, VarId> = make_id_mapping(field);

    let tent_mapping: HashMap<VarId, TentPlace> = id_mapping
        .iter()
        .map(|(coord, id)| (*id, *coord))
        .collect::<HashMap<VarId, TentPlace>>();

    let col_set: AxisSet = make_coord_set_by(&|x: &TentPlace| -> usize { x.1 }, &tent_mapping);
    let row_set: AxisSet = make_coord_set_by(&|x: &TentPlace| -> usize { x.0 }, &tent_mapping);
    let neighbour_set: NeighbourSet = make_neighbour_set(&tent_mapping, &id_mapping);

    let mut total = CNF::empty();
    let ((count_col_form, count_row_form), (neigs_form, (corr_form, _assg_mapping))) = rayon::join(
        || {
            rayon::join(
                || make_count_constraints(&field.column_counts, &col_set),
                || make_count_constraints(&field.row_counts, &row_set),
            )
        },
        || {
            rayon::join(
                || make_neighbour_constraints(&neighbour_set),
                || make_correspondence_constraints(&field, &id_mapping),
            )
        },
    );
    total.extend(count_col_form);
    total.extend(count_row_form);
    total.extend(neigs_form);
    total.extend(corr_form);
    (total, id_mapping)
}

/// For each coordinate (extracted by the `by` function) describes
/// a vector of tent places that lie on it
fn make_coord_set_by(
    by: impl Fn(&TentPlace) -> usize,
    tents: &HashMap<usize, TentPlace>,
) -> AxisSet {
    let mut out: AxisSet = HashMap::new();

    for (id, coord) in tents {
        out.entry(by(coord))
            .and_modify(|v| v.push(*id))
            .or_insert(vec![*id]);
    }
    out
}

/// Creates collection of pairs of adjacent tent places
fn make_neighbour_set(
    id_to_coord: &HashMap<usize, TentPlace>,
    coord_to_id: &HashMap<TentPlace, usize>,
) -> NeighbourSet {
    let mut out: NeighbourSet = Vec::new();

    for (id, (row, column)) in id_to_coord {
        let mut neighbours = Vec::new();
        let mut candidates = Vec::new();
        let row = *row;
        let column = *column;

        candidates.push((row + 1, column));
        candidates.push((row, column + 1));

        if column > 0 {
            candidates.push((row + 1, column - 1))
        }
        candidates.push((row + 1, column + 1));

        for candidate in candidates {
            if coord_to_id.contains_key(&candidate) {
                neighbours.push((*id, coord_to_id[&candidate]));
            }
        }

        out.extend(neighbours.into_iter());
    }

    out
}

/// Builds up a formula that forces certain number of present tents in
/// a given row or column
pub fn axis_constraint(variables: &Vec<usize>, count: usize) -> CNF {
    let lower_bound_clauses = variables
        .iter()
        .combinations(variables.len() - count + 1)
        .par_bridge()
        .map(|vs| {
            vs.iter()
                .map(|v| CNFVar::pos(**v as VarId))
                .collect::<CNFClause>()
        });

    let upper_bound_clauses = variables
        .iter()
        .combinations(count + 1)
        .par_bridge()
        .map(|vs| {
            vs.iter()
                .map(|v| CNFVar::neg(**v as VarId))
                .collect::<CNFClause>()
        });

    lower_bound_clauses
        .chain(upper_bound_clauses)
        .collect::<CNF>()
}

/// Builds up a formula that forces certain number of present tents in
/// all rows or columns
fn make_count_constraints(counts: &Vec<usize>, axes: &AxisSet) -> CNF {
    let mut clauses = CNF::empty();

    // Conjunction of constranits per each axis
    for (axis, tents) in axes {
        let axis_count = counts[*axis];
        if axis_count == 0 {
            continue;
        }
        let for_axis = axis_constraint(tents, axis_count);
        clauses.extend(for_axis);
    }
    clauses
}

/// Builds up a formula that bans tents placed on adjacent positions
fn make_neighbour_constraints(neigh: &NeighbourSet) -> CNF {
    neigh
        .iter()
        .map(|(x, y)| CNFClause {
            vars: vec![CNFVar::neg(*x as VarId), CNFVar::neg(*y as VarId)],
        })
        .collect()
}

/// Here are the dragons. Builds up a formula that asserts a bijection between
/// trees and tents
pub fn make_correspondence_constraints(
    field: &Field,
    id_mapping: &HashMap<TentPlace, usize>,
) -> (CNF, Vec<(Assignment, usize)>) {
    let mut oneof_packs: Vec<Vec<Assignment>> = vec![];
    let mut cond_oneof_packs: Vec<(TentPlace, Vec<Assignment>)> = vec![];

    let mut assignments: Vec<Assignment> = vec![];
    for row in 0..field.height {
        for column in 0..field.width {
            let cell = field.get_cell(row, column);
            if cell == CellType::Tree || cell == CellType::Meadow || cell == CellType::Tent {
                let is_tree = cell == CellType::Tree;
                let looking_for = |t| {
                    if is_tree {
                        t == CellType::Meadow || t == CellType::Tent
                    } else {
                        t == CellType::Tree
                    }
                };

                let mut pack: Vec<Assignment> = vec![];

                if let Some(left) = column.checked_sub(1) {
                    if looking_for(field.get_cell(row, left)) {
                        let asg = if is_tree {
                            Assignment {
                                tent: (row, left),
                                tree: (row, column),
                            }
                        } else {
                            Assignment {
                                tree: (row, left),
                                tent: (row, column),
                            }
                        };
                        pack.push(asg);
                        assignments.push(asg);
                    }
                }

                let right = column + 1;
                if right < field.width && looking_for(field.get_cell(row, right)) {
                    let asg = if is_tree {
                        Assignment {
                            tent: (row, right),
                            tree: (row, column),
                        }
                    } else {
                        Assignment {
                            tree: (row, right),
                            tent: (row, column),
                        }
                    };
                    pack.push(asg);
                    assignments.push(asg);
                }

                if let Some(top) = row.checked_sub(1) {
                    if looking_for(field.get_cell(top, column)) {
                        let asg = if is_tree {
                            Assignment {
                                tent: (top, column),
                                tree: (row, column),
                            }
                        } else {
                            Assignment {
                                tree: (top, column),
                                tent: (row, column),
                            }
                        };
                        pack.push(asg);
                        assignments.push(asg);
                    }
                }

                let bottom = row + 1;
                if bottom < field.height && looking_for(field.get_cell(bottom, column)) {
                    let asg = if is_tree {
                        Assignment {
                            tent: (bottom, column),
                            tree: (row, column),
                        }
                    } else {
                        Assignment {
                            tree: (bottom, column),
                            tent: (row, column),
                        }
                    };
                    pack.push(asg);
                    assignments.push(asg);
                }

                if pack.len() != 0 {
                    if is_tree {
                        oneof_packs.push(pack);
                    } else {
                        cond_oneof_packs.push(((row, column), pack));
                    }
                }
            }
        }
    }

    assignments.sort();
    assignments.dedup();

    let assg_mapping = assignments
        .iter()
        .unique()
        .enumerate()
        .map(|(id, coord)| (id + id_mapping.len() + 1, *coord))
        .collect::<HashMap<usize, Assignment>>();

    let assg_id_mapping = assg_mapping
        .iter()
        .map(|(id, coord)| (*coord, *id))
        .collect::<HashMap<Assignment, usize>>();

    fn makeoneof(
        id_mapping: &HashMap<TentPlace, usize>,
        assg_id_mapping: &HashMap<Assignment, usize>,
        pack: &Vec<Assignment>,
        cond: Option<TentPlace>,
    ) -> Vec<CNFClause> {
        let ids: Vec<&usize> = pack
            .iter()
            .map(|assg| assg_id_mapping.get(assg).unwrap())
            .collect();
        let mut any_of = vec![match cond {
            Some(x) => {
                let mut c = ids
                    .iter()
                    .map(|i| CNFVar::pos(**i as VarId))
                    .collect::<CNFClause>();
                c.push(CNFVar::neg(*id_mapping.get(&x).unwrap() as VarId));
                c
            }
            None => ids
                .iter()
                .map(|i| CNFVar::pos(**i as VarId))
                .collect::<CNFClause>(),
        }];

        let no_two = {
            let mut out = vec![];
            for i in &ids {
                for j in &ids {
                    if i < j {
                        out.push(match cond {
                            Some(x) => {
                                let mut c = CNFClause::new();
                                c.push(CNFVar::neg(**i as VarId));
                                c.push(CNFVar::neg(**j as VarId));
                                c.push(CNFVar::neg(*id_mapping.get(&x).unwrap() as VarId));
                                c
                            }
                            None => {
                                let mut c = CNFClause::new();
                                c.push(CNFVar::neg(**i as VarId));
                                c.push(CNFVar::neg(**j as VarId));
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

    let pack_formula = oneof_packs
        .par_iter()
        .flat_map(|p| makeoneof(&id_mapping, &assg_id_mapping, p, None));

    let cond_pack_formula = cond_oneof_packs
        .par_iter()
        .flat_map(|(cond, p)| makeoneof(&id_mapping, &assg_id_mapping, p, Some(*cond)));

    let tent_exists_formula = assignments.par_iter().map(|a| CNFClause {
        vars: vec![
            CNFVar::neg(*assg_id_mapping.get(a).unwrap() as VarId),
            CNFVar::pos(*id_mapping.get(&a.tent).unwrap() as VarId),
        ],
    });

    (
        pack_formula
            .chain(cond_pack_formula)
            .chain(tent_exists_formula)
            .collect(),
        assg_id_mapping
            .iter()
            .map(|(a, i)| (*a, *i))
            .collect::<Vec<(Assignment, usize)>>(),
    )
}
