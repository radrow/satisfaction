use std::fmt;
use crate::cnf::*;
use std::collections::VecDeque;

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

        fn go(form : &Formula, mut clauses : &mut VecDeque<CNFClause>, neg : bool) {
            match form {
                Formula::Const(b) => {
                    if neg == *b {
                        clauses.push_back(CNFClause{vars: vec![]});
                    }
                },
                Formula::Var(v) => {
                    if neg {
                        clauses.push_back(CNFClause{vars: vec![CNFVar::Neg(String::from(v.clone()))]});
                    } else {
                        clauses.push_back(CNFClause{vars: vec![CNFVar::Pos(String::from(v.clone()))]});
                    }
                },
                Formula::And(l, r) => {
                    if neg {
                        let mut clauses_l : VecDeque<CNFClause> = VecDeque::new();
                        let mut clauses_r : VecDeque<CNFClause> = VecDeque::new();
                        go(l, &mut clauses_l, true);
                        go(r, &mut clauses_r, true);
                        for cl in &clauses_l {
                            for cr in &clauses_r {
                                clauses.push_back(cl.cat(&cr))
                            }
                        }
                    } else {
                        go(l, &mut clauses, false);
                        go(r, &mut clauses, false);
                    }
                },
                Formula::Or(l, r) => {
                    if neg {
                        go(l, &mut clauses, true);
                        go(r, &mut clauses, true);
                    } else {
                        let mut clauses_l : VecDeque<CNFClause> = VecDeque::new();
                        let mut clauses_r : VecDeque<CNFClause> = VecDeque::new();
                        go(l, &mut clauses_l, false);
                        go(r, &mut clauses_r, false);
                        for cl in &clauses_l {
                            for cr in &clauses_r {
                                clauses.push_back(cl.cat(&cr))
                            }
                        }
                    }
                },
                Formula::Not(p) => go(p, &mut clauses, true),
            }
        }

        go(self, &mut out_clauses, false);
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
