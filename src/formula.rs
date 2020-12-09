use std::fmt;
use crate::cnf::*;

#[derive (Clone)]
pub enum Formula {
    Const(bool),
    Var(String),
    Not(Box<Formula>),
    Or(Box<Formula>, Box<Formula>),
    And(Box<Formula>, Box<Formula>),
    Imp(Box<Formula>, Box<Formula>),
    Iff(Box<Formula>, Box<Formula>),
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

    pub fn imp(self, other : Formula) -> Formula {
        match (&self, &other) {
            (Formula::Var(v1), Formula::Var(v2)) if v1 == v2 => Formula::Const(true),
            (Formula::Const(false), _) => Formula::Const(true),
            (_, Formula::Const(true)) => Formula::Const(true),
            (Formula::Const(true), Formula::Const(false)) => Formula::Const(false),
            _ => Formula::Imp(Box::new(self), Box::new(other))
        }
    }

    pub fn iff(self, other : Formula) -> Formula {
        match (&self, &other) {
            (Formula::Var(v1), Formula::Var(v2)) if v1 == v2 => Formula::Const(true),
            (Formula::Const(true), Formula::Const(false)) => Formula::Const(false),
            (Formula::Const(false), Formula::Const(true)) => Formula::Const(false),
            (Formula::Const(true), Formula::Const(true)) => Formula::Const(true),
            (Formula::Const(false), Formula::Const(false)) => Formula::Const(true),
            _ => Formula::Iff(Box::new(self), Box::new(other))
        }
    }

    pub fn to_cnf(&self) -> CNF {
        match self {
            Formula::Const(true) => CNF{clauses: vec![]},
            Formula::Const(false) => CNF{clauses: vec![CNFClause{vars: vec![]}]},
            Formula::Var(v) =>
                CNF{clauses: vec![CNFClause{vars: vec![CNFVar::Pos(String::from(v.clone()))]}]},
            Formula::And(l, r) =>
                l.to_cnf().cat(&r.to_cnf()),
            Formula::Or(l, r) => {
                let mut clauses = vec![];
                for cl in l.to_cnf().clauses {
                    for cr in r.to_cnf().clauses {
                        clauses.push(cl.cat(&cr))
                    }
                }
                CNF{clauses: clauses}
            },
            Formula::Not(p) =>
                match &**p {
                    Formula::Const(true) => CNF{clauses: vec![CNFClause{vars: vec![]}]},
                    Formula::Const(false) => CNF{clauses: vec![]},
                    Formula::Var(v) =>
                        CNF{clauses: vec![CNFClause{vars: vec![CNFVar::Neg(String::from(v.clone()))]}]},
                    Formula::And(l, r) =>
                        l.clone().not().or(r.clone().not()).to_cnf(),
                    Formula::Or(l, r) =>
                        l.clone().not().and(r.clone().not()).to_cnf(),
                    Formula::Not(q) => q.to_cnf(),
                    Formula::Imp(l, r) =>
                        l.clone().and(r.clone().not()).to_cnf(),
                    Formula::Iff(l, r) =>
                        (l.clone().not().and(*r.clone()))
                        .or(r.clone().not().and(*l.clone()))
                        .to_cnf()
                },
            Formula::Imp(l, r) =>
                l.clone().not().or(*r.clone()).to_cnf(),
            Formula::Iff(l, r) =>
                (l.clone().and(*r.clone()))
                .or(l.clone().not().and(r.clone().not()))
                .to_cnf()
        }
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
            Formula::Imp(l, r) => write!(f, "{} => {}", l, r),
            Formula::Iff(l, r) => write!(f, "({}) <=> ({})", l, r),
        }
    }
}
