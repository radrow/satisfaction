use std::fmt;
use crate::cnf::*;
use std::collections::VecDeque;

#[derive (Clone)]
pub enum Formula {
    Const(bool),
    Var(String),
    Not(Box<Formula>),
    Or(Box<Formula>, Box<Formula>),
    And(Box<Formula>, Box<Formula>),
}

impl Formula {
    pub fn not(self) -> Formula {
        match &self {
            Formula::Const(true) => Formula::Const(false),
            Formula::Const(false) => Formula::Const(true),
            _ => Formula::Not(Box::new(self))
        }
    }

    pub fn or(self, other : Formula) -> Formula {
        match (&self, &other) {
            (Formula::Var(v1), Formula::Var(v2)) if v1 == v2 => self,
            (Formula::Const(true), _) => Formula::Const(true),
            (_, Formula::Const(true)) => Formula::Const(true),
            (Formula::Const(false), Formula::Const(false)) => Formula::Const(false),
            _ => Formula::Or(Box::new(self), Box::new(other))
        }
    }

    pub fn and(self, other : Formula) -> Formula {
        match (&self, &other) {
            (Formula::Var(v1), Formula::Var(v2)) if v1 == v2 => self,
            (Formula::Const(false), _) => Formula::Const(false),
            (_, Formula::Const(false)) => Formula::Const(false),
            (Formula::Const(true), Formula::Const(true)) => Formula::Const(true),
            _ => Formula::And(Box::new(self), Box::new(other))
        }
    }

    pub fn to_cnf(&self) -> CNF {
        let mut out_clauses : VecDeque<CNFClause> = VecDeque::new();

        fn go(form : &Formula, mut clauses : &mut VecDeque<CNFClause>) {
            match form {
                Formula::Const(true) => (),
                Formula::Const(false) => {
                    clauses.push_back(CNFClause{vars: vec![]});
                },
                Formula::Var(v) => {
                    clauses.push_back(CNFClause{vars: vec![CNFVar::Pos(String::from(v.clone()))]});
                },
                Formula::And(l, r) => {
                    go(l, &mut clauses);
                    go(r, &mut clauses);
                },
                Formula::Or(l, r) => {
                    let mut clauses_l : VecDeque<CNFClause> = VecDeque::new();
                    let mut clauses_r : VecDeque<CNFClause> = VecDeque::new();
                    go(l, &mut clauses_l);
                    go(r, &mut clauses_r);
                    for cl in &clauses_l {
                        for cr in &clauses_r {
                            clauses.push_back(cl.cat(&cr))
                        }
                    }
                },
                Formula::Not(p) =>
                    match &**p {
                        Formula::Const(true) => {
                            clauses.push_back(CNFClause{vars: vec![]});
                        },
                        Formula::Const(false) => (),
                        Formula::Var(v) => {
                            clauses.push_back(CNFClause{vars: vec![CNFVar::Neg(String::from(v.clone()))]});
                        },
                    Formula::And(l, r) => {
                        go(&l.clone().not().or(r.clone().not()), &mut clauses);
                    },
                        Formula::Or(l, r) => {
                            go(&l.clone().not().and(r.clone().not()), &mut clauses);
                        },
                        Formula::Not(q) => {
                            go(q, &mut clauses);
                        }
                    },
        }
        }

        go(self, &mut out_clauses);
        CNF{clauses: out_clauses}
    }

}

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Formula::Const(true) => write!(f, "1"),
            Formula::Const(false) => write!(f, "0"),
            Formula::Var(v) => write!(f, "{}", v),
            Formula::Not(l) => write!(f, "~{}", l),
            Formula::Or(l, r) => write!(f, "({}) \\/ ({})", l, r),
            Formula::And(l, r) => write!(f, "({}) /\\ ({})", l, r),
        }
    }
}
