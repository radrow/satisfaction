use std::fmt;
use std::iter::FromIterator;
use std::collections::HashSet;

use rayon::iter::*;
use dimacs::parse_dimacs;

use cadical;

pub type VarId = usize;

#[derive(Clone, Debug)]
pub struct CNF {
    pub clauses: Vec<CNFClause>,
    pub num_variables: usize,
}

#[derive(Clone, Debug)]
pub struct CNFClause {
    pub vars: Vec<CNFVar>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct CNFVar {
    pub id: VarId,
    pub sign: bool,
}

impl CNF {
    pub fn empty() -> CNF {
        CNF{
            clauses: Vec::new(), 
            num_variables: 0,
        }
    }

    pub fn single(clause : CNFClause) -> CNF {
        CNF {
            num_variables: clause.max_variable_id(),
            clauses: vec![clause],
        }
    }

    pub fn push(&mut self, c: CNFClause) {
        self.num_variables = self.num_variables.max(c.max_variable_id());
        self.clauses.push(c);
    }

    pub fn extend(&mut self, c: CNF) {
        if let Some(max_variable_id) = c.clauses
            .iter()
                .map(|clause| clause.max_variable_id())
                .max() {
            self.num_variables = self.num_variables.max(max_variable_id);
            self.clauses.extend(c.clauses)
        }
    }

    #[allow(dead_code)]
    pub fn to_dimacs(&self) -> String {
        let mut out : String = String::from("");

        let distinct : HashSet<VarId> = self.clauses.iter()
            .flat_map(|clause| clause.vars.iter().map(|v| v.id()))
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

    pub fn from_dimacs(input : &str) -> Result<CNF, dimacs::ParseError> {
        let inst = parse_dimacs(input);

        match inst {
            Ok(dimacs::Instance::Cnf{clauses, num_vars}) => {
                let clauses = clauses.iter()
                    .map(|clause| {
                        clause.lits()
                            .iter()
                            .map(|lit| CNFVar {
                                id: lit.var().to_u64() as VarId,
                                sign: lit.sign() == dimacs::Sign::Pos,
                            }).collect()
                    }).collect();
                Ok(CNF {
                    clauses,
                    num_variables: num_vars as usize,
                })
            },
            // TODO: Better error handling
            Ok(_) => panic!("was zum kuh"),
            Err(e) => Err(e)
        }
    }
}

impl FromParallelIterator<CNFClause> for CNF {
    fn from_par_iter<I : IntoParallelIterator<Item=CNFClause>>(iter: I) -> Self {
        let clauses = iter.into_par_iter()
                .collect::<Vec<CNFClause>>();
        let num_variables = clauses.iter()
            .map(|clause| clause.max_variable_id())
            .max()
            .unwrap_or(0);

        CNF {
            clauses,
            num_variables,
        }
    }
}

impl FromIterator<CNFClause> for CNF {
    fn from_iter<I: IntoIterator<Item=CNFClause>>(iter: I) -> Self {
        let clauses = iter.into_iter()
                .collect::<Vec<CNFClause>>();

        let num_variables = clauses.iter()
            .map(|clause| clause.max_variable_id())
            .max()
            .unwrap_or(0);

        CNF {
            clauses,
            num_variables,
        }
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

    pub fn max_variable_id(&self) -> usize {
        self.vars.iter()
            .map(|lit| lit.id)
            .max()
            .unwrap_or(0)
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
    pub fn new(id: VarId, sign: bool) -> CNFVar {
        CNFVar{id, sign}
    }

    pub fn pos(id: VarId) -> CNFVar {
        CNFVar{id, sign: true}
    }

    pub fn neg(id: VarId) -> CNFVar{
        CNFVar{id, sign: false}
    }

    pub fn id(&self) -> VarId {
        self.id
    }

    pub fn sign(&self) -> bool {
        self.sign
    }

    pub fn to_i32(&self) -> i32 {
        if self.sign {
            self.id as i32
        } else {
            -(self.id as i32)
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
        write!(f, "{}", self.to_i32())
    }
}
