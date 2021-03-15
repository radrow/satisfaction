use std::{collections::BinaryHeap, cmp::Reverse, ops::{Index, IndexMut}, iter::FromIterator};
use itertools::Itertools;
use stable_vec::StableVec;
use crate::{CNFVar, CNFClause};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::fmt;

pub type ClauseId = usize;

#[derive(Debug)]
pub struct Clause {
    pub literals: Vec<CNFVar>,
    pub watched_literals: [usize; 2],
}

impl Clause {
    pub fn new(cnf_clause: &CNFClause) -> Clause {
        // decrement the variables by 1 to get a 0 offset
        let mut cnf_variables = cnf_clause.vars.clone();
        cnf_variables.iter_mut().for_each(|var| var.id -= 1);

        // assign the first and the last literal as watched literals
        let mut watched_literals: [usize; 2] = [0, 0];
        if cnf_variables.len() > 0 {
            watched_literals = [0, cnf_variables.len() - 1];
        }

        Clause {
            literals: cnf_variables,
            watched_literals,
        }
    }

    pub fn get_watched_lits(&self) -> (CNFVar, CNFVar) {
        (self.literals[self.watched_literals[0]], self.literals[self.watched_literals[1]])
    }

    pub fn get_first_watched(&self) -> CNFVar {
        self.literals[self.watched_literals[0]]
    }

    #[allow(dead_code)]
    pub fn get_second_watched(&self) -> CNFVar {
        self.literals[self.watched_literals[1]]
    }
}

pub struct Clauses {
    formula: Vec<Clause>,
    additional_clauses: StableVec<Clause>,
    used_indices: BinaryHeap<Reverse<usize>>,
    drup: Option<DrupListener>,
}

impl Clauses {
    pub fn new(formula: Vec<Clause>) -> Clauses {
        Clauses {
            formula,
            additional_clauses: StableVec::new(),
            used_indices: BinaryHeap::new(),
            drup: None,
        }
    }

    pub fn drup(formula: Vec<Clause>, path: impl AsRef<Path>) -> Clauses {
        Clauses {
            formula,
            additional_clauses: StableVec::new(),
            used_indices: BinaryHeap::new(),
            drup: Some(DrupListener::new(path)),
        }
    }

    /// Expects that the first literal in clause is free.
    pub fn push(&mut self, clause: CNFClause) -> usize {
        let mut watched_literals: [usize; 2] = [0, 0];
        if clause.len() > 0 {
            watched_literals = [0, clause.len() - 1];
        }

        let clause = Clause {
            literals: clause.vars,
            watched_literals,
        };

        if let Some(drup) = &mut self.drup {
            drup.learn(&clause);
        }

        self.formula.len() + if let Some(Reverse(index)) = self.used_indices.pop() {
            self.additional_clauses.insert(index, clause);
            index
        } else {
            self.additional_clauses.push(clause)
        }
    }

    pub fn len(&self) -> usize {
        self.formula.len() + self.additional_clauses.num_elements()
    }

    pub fn len_formula(&self) -> usize {
        self.formula.len()
    }

    pub fn remove(&mut self, index: usize) -> (CNFVar, CNFVar) {
        let index = index.checked_sub(self.formula.len())
            .expect("Cannot remove clauses from the original formula!");

        self.used_indices.push(Reverse(index));

        self.additional_clauses.remove(index)
            .expect("Clause to delete was already deleted!")
            .get_watched_lits()
    }

    pub fn iter(&self) -> impl std::iter::Iterator<Item=&Clause> {
        self.formula.iter().chain(self.additional_clauses.values())
    }
}

impl Index<ClauseId> for Clauses {
    type Output = Clause;
    fn index(&self, index: ClauseId) -> &Self::Output {
        if index < self.formula.len() {
            &self.formula[index]
        } else {
            &self.additional_clauses[index-self.formula.len()]
        }
    }
}

impl IndexMut<ClauseId> for Clauses {
    fn index_mut(&mut self, index: ClauseId) -> &mut Self::Output {
        if index < self.formula.len() {
            &mut self.formula[index]
        } else {
            let l = self.formula.len();
            &mut self.additional_clauses[index-l]
        }
    }
}

impl FromIterator<Clause> for Clauses {
    fn from_iter<T: IntoIterator<Item=Clause>>(iter: T) -> Self {
        Clauses::new(iter.into_iter().collect())
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.literals.iter().map(|lit| format!("{}{}", if lit.sign { ' ' } else { '-' }, lit.id+1)).join(" "))
    }
}


struct DrupListener(File);

impl DrupListener {
    fn new(path: impl AsRef<Path>) -> DrupListener {
        let file = File::create(path)
            .unwrap();
        DrupListener(file)
    }

    fn learn(&mut self, clause: &Clause) {
        self.0.write_all(format!("{} 0\n", clause).as_bytes())
            .unwrap();
    }

    #[allow(dead_code)]
    fn delete(&mut self, clause: &Clause) {
        self.0.write_all(format!("d {} 0\n", clause).as_bytes())
            .unwrap();
    }
}

impl Drop for DrupListener {
    fn drop(&mut self) {
        self.0.write("0".as_bytes())
            .unwrap();
    }
}
