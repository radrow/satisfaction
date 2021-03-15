use crate::CNF;
use super::clause::ClauseId;
use super::util::HashSet;


pub type VariableId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentType {
    Forced(ClauseId),
    Branching,
    Known,
}

#[derive(Debug, Clone, Copy)]
pub struct Assignment {
    pub sign: bool,
    pub branching_level: usize,
    pub reason: AssignmentType,
}


#[derive(Debug, Clone)]
pub struct Variable {
    pub watched_occ: HashSet<ClauseId>,
    pub assignment: Option<Assignment>,
}

impl Variable {
    pub fn new(cnf: &CNF, var_num: usize) -> Variable {
        // default assignment if the variable is not contained in any clause and is empty
        let mut assignment: Option<Assignment> = Some(Assignment {
            sign: false,
            branching_level: 0,
            reason: AssignmentType::Known,
        });
        // if variable is contained in any clause set it to unassigned first
        cnf.clauses.iter().for_each(|clause| {
            for var in &clause.vars {
                if var_num == var.id {
                    assignment = None;
                }
            }
        });

        Variable {
            watched_occ: cnf.clauses
                .iter()
                .enumerate()
                .filter_map(|(index, clause)| {
                    if clause.vars.first()?.id == var_num {
                        return Some(index);
                    }
                    if clause.vars.last()?.id == var_num {
                        return Some(index);
                    }
                    return None;
                }).collect(),
            assignment 
        }
    }

    pub fn add_watched_occ(&mut self, index: ClauseId) {
        self.watched_occ.insert(index);
    }

    pub fn remove_watched_occ(&mut self, index: ClauseId) {
        self.watched_occ.remove(&index);
    }
}


pub type Variables = Vec<Variable>;
