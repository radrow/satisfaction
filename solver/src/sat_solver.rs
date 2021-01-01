use crate::CNFClause;

pub type Assignment = Vec<bool>;

pub trait Solver {
    fn solve(&self, clauses: impl Iterator<Item=CNFClause>, num_variables: usize) -> Option<Assignment>;
}
