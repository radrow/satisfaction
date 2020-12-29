use std::fmt;
use std::iter::FromIterator;
use std::collections::HashSet;

use cadical;

pub struct CNF {
    pub clauses : Vec<CNFClause>
}

pub struct CNFClause {
    pub vars : Vec<CNFVar>
}

pub enum CNFVar {
    Pos(u32),
    Neg(u32)
}

impl CNF {
    pub fn new() -> CNF {
        CNF{clauses: vec![]}
    }

    pub fn single(clause : CNFClause) -> CNF {
        CNF{clauses: vec![clause]}
    }

    pub fn push(&mut self, c : CNFClause) {
        self.clauses.push(c)
    }

    pub fn extend(&mut self, c : CNF) {
        self.clauses.extend(c.clauses)
    }

    #[allow(dead_code)]
    pub fn to_dimacs(&self) -> String {
        let mut out : String = String::from("");

        let distinct : HashSet<u32> =
            self.clauses.iter().flat_map(|clause| clause.vars.iter().map(|v| v.name()))
            .collect();

        out.extend("p cnf ".chars());
        out.extend(distinct.len().to_string().chars());
        out.extend(" ".chars());
        out.extend(self.clauses.len().to_string().chars());
        out.extend("\n".chars());

        for clause in &self.clauses {
            for var in &clause.vars {
                out.extend(var.to_i32().to_string().chars());
                out.extend(" ".chars())
            }
            out.extend("0\n".chars())
        }
        out
    }

    pub fn to_solver(&self) -> cadical::Solver {
        let mut sat: cadical::Solver = Default::default();

        for clause in &self.clauses {
            sat.add_clause(clause.vars.iter().map(|var| var.to_i32()));
        }

        sat
    }
}

impl FromIterator<CNFClause> for CNF {
    fn from_iter<I: IntoIterator<Item=CNFClause>>(iter: I) -> Self {
        CNF{clauses: iter.into_iter().collect()}
    }
}

impl IntoIterator for CNF {
    type Item = CNFClause;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.clauses.into_iter()
    }
}

impl CNFClause {
    pub fn new() -> CNFClause {
        CNFClause{vars: vec![]}
    }

    #[allow(dead_code)]
    pub fn single(var : CNFVar) -> CNFClause {
        CNFClause{vars: vec![var]}
    }

    pub fn push(&mut self, v : CNFVar) {
        self.vars.push(v)
    }

    #[allow(dead_code)]
    pub fn extend(&mut self, c : CNFClause) {
        self.vars.extend(c.vars)
    }
}

impl FromIterator<CNFVar> for CNFClause {
    fn from_iter<I: IntoIterator<Item=CNFVar>>(iter: I) -> Self {
        CNFClause{vars: iter.into_iter().collect()}
    }
}

impl IntoIterator for CNFClause {
    type Item = CNFVar;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.vars.into_iter()
    }
}

impl CNFVar {
    pub fn name(&self) -> u32 {
        match self {
            CNFVar::Pos(s) => *s,
            CNFVar::Neg(s) => *s,
        }
    }
    pub fn to_i32(&self) -> i32 {
        match self {
            CNFVar::Pos(s) => (*s as i32),
            CNFVar::Neg(s) => -(*s as i32),
        }
    }
}


impl fmt::Display for CNF {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in &self.clauses {
            write!(f, "{}\n", c)?;
        }
        write!(f, "")
    }
}
impl fmt::Display for CNFClause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in &self.vars {
            write!(f, "({})  ", c)?;
        }
        write!(f, "")
    }
}
impl fmt::Display for CNFVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CNFVar::Pos(s) =>
                write!(f, "{}", s),
            CNFVar::Neg(s) =>
                write!(f, "!{}", s),
        }
    }
}
