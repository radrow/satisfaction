use itertools::Itertools;
use std::collections::HashSet;
use std::fmt;
use std::iter::FromIterator;

use dimacs::parse_dimacs;
use rayon::iter::*;

/// Type used for referencing logical variables
pub type VarId = usize;

/// Representation of logical formulae in CNF form
/// (conjunction of clausees)>
#[derive(Clone, Debug)]
pub struct CNF {
    /// Vector of inner clauses
    pub clauses: Vec<CNFClause>,
    pub num_variables: usize,
}

/// Representation of a clause (disjunction of variables)
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct CNFClause {
    /// Vector of inner variables
    pub vars: Vec<CNFVar>,
}

/// Logical variable
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct CNFVar {
    /// Identifier of a variable
    pub id: VarId,
    /// Variable is negated iff `sign == false`
    pub sign: bool,
}

impl CNF {
    /// Creates an empty CNF formula
    #[inline]
    pub fn empty() -> CNF {
        CNF {
            clauses: Vec::new(),
            num_variables: 0,
        }
    }

    /// Creates a singleton CNF formula out of a single clause
    #[inline]
    pub fn single(clause: CNFClause) -> CNF {
        CNF {
            num_variables: clause.max_variable_id(),
            clauses: vec![clause],
        }
    }

    /// Adds a new clause to the formula
    #[inline]
    pub fn push(&mut self, c: CNFClause) {
        self.num_variables = self.num_variables.max(c.max_variable_id());
        self.clauses.push(c);
    }

    pub fn extend(&mut self, c: CNF) {
        if let Some(max_variable_id) = c
            .clauses
            .iter()
            .map(|clause| clause.max_variable_id())
            .max()
        {
            self.num_variables = self.num_variables.max(max_variable_id);
            self.clauses.extend(c.clauses)
        }
    }

    /// Returns number of clauses in the formula
    #[inline]
    pub fn len(&self) -> usize {
        self.clauses.len()
    }

    /// Collects all variable identifiers that appear in the formula
    #[inline]
    pub fn vars(&self) -> HashSet<VarId> {
        self.clauses
            .iter()
            .flat_map(|clause| clause.vars.iter().map(CNFVar::id))
            .unique()
            .collect()
    }

    /// Prints formula in DIMACS compatible form
    pub fn to_dimacs(&self) -> String {
        let mut out: String = String::from("");

        let distinct = self.vars();

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

    /// Parse DIMACS string into CNF structure
    pub fn from_dimacs(input: &str) -> Result<CNF, String> {
        let inst = parse_dimacs(input);

        match inst {
            Ok(dimacs::Instance::Cnf { clauses, num_vars }) => {
                let clauses = clauses
                    .iter()
                    .map(|clause| {
                        clause
                            .lits()
                            .iter()
                            .map(|lit| CNFVar {
                                id: lit.var().to_u64() as VarId,
                                sign: lit.sign() == dimacs::Sign::Pos,
                            })
                            .collect()
                    })
                    .collect();
                Ok(CNF {
                    clauses,
                    num_variables: num_vars as usize,
                })
            }
            Ok(_) => Err("Only CNF formulae are supported".to_string()),
            Err(_) => Err("Parse error".to_string()),
        }
    }
}

impl FromParallelIterator<CNFClause> for CNF {
    fn from_par_iter<I: IntoParallelIterator<Item = CNFClause>>(iter: I) -> Self {
        let clauses = iter.into_par_iter().collect::<Vec<CNFClause>>();
        let num_variables = clauses
            .iter()
            .map(|clause| clause.max_variable_id())
            .max()
            .unwrap_or(0);

        CNF {
            clauses,
            num_variables,
        }
    }
}

impl IntoParallelIterator for CNF {
    type Item = CNFClause;
    type Iter = rayon::vec::IntoIter<CNFClause>;

    fn into_par_iter(self) -> Self::Iter {
        self.clauses.into_par_iter()
    }
}

impl FromIterator<CNFClause> for CNF {
    fn from_iter<I: IntoIterator<Item = CNFClause>>(iter: I) -> Self {
        let clauses = iter.into_iter().collect::<Vec<CNFClause>>();

        let num_variables = clauses
            .iter()
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
    /// Creates an empty CNF clause
    #[inline]
    pub fn new() -> CNFClause {
        CNFClause { vars: Vec::new() }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> CNFClause {
        CNFClause { vars: Vec::with_capacity(capacity) }
    }

    /// Creates a CNF clause containing a single variable
    #[inline]
    pub fn single(var: CNFVar) -> CNFClause {
        CNFClause { vars: vec![var] }
    }

    /// Adds a single variable into the clause
    #[inline]
    pub fn push(&mut self, v: CNFVar) {
        match self.vars.binary_search(&v) {
            Ok(_) => {} // we could check for contradiction here
            Err(pos) => self.vars.insert(pos, v),
        }
    }

    /// Returns the greatest variablie identifier used in the clause
    #[inline]
    pub fn max_variable_id(&self) -> usize {
        self.vars.iter().map(|lit| lit.id).max().unwrap_or(0)
    }

    /// Concatenates clauses
    #[inline]
    pub fn extend(&mut self, c: CNFClause) {
        self.vars.reserve(self.len() + c.len());
        for v in c {
            self.push(v)
        }
    }

    /// Calculates number of literals in clause
    #[inline]
    pub fn len(&self) -> usize {
        self.vars.len()
    }
}

impl FromIterator<CNFVar> for CNFClause {
    fn from_iter<I: IntoIterator<Item = CNFVar>>(iter: I) -> Self {
        CNFClause {
            vars: iter.into_iter().collect(),
        }
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
    /// Creates variable with given identifier and positivity
    #[inline]
    pub fn new(id: VarId, sign: bool) -> CNFVar {
        CNFVar { id, sign }
    }

    /// Creates a positive variable with given identifier
    #[inline]
    pub fn pos(id: VarId) -> CNFVar {
        CNFVar { id, sign: true }
    }

    /// Creates a negative variable with given identifier
    #[inline]
    pub fn neg(id: VarId) -> CNFVar {
        CNFVar { id, sign: false }
    }

    /// Gets the identifier of a variable
    #[inline]
    pub fn id(&self) -> VarId {
        self.id
    }

    /// Checks if the variable is positive
    #[inline]
    pub fn sign(&self) -> bool {
        self.sign
    }

    /// Converts to signed integer. The absolute value indicates
    /// the identifier and sign states for positivity.
    ///
    /// **NOTE** it is not integer-overflow friendly.
    pub fn to_i32(&self) -> i32 {
        if self.sign {
            self.id as i32
        } else {
            -(self.id as i32)
        }
    }
}

impl std::ops::Neg for CNFVar {
    type Output = CNFVar;
    fn neg(mut self) -> Self::Output {
        self.sign = !self.sign;
        self
    }
}

impl From<CNFVar> for (VarId, bool) {
    fn from(lit: CNFVar) -> Self {
        (lit.id, lit.sign)
    }
}

impl From<&CNFVar> for (VarId, bool) {
    fn from(lit: &CNFVar) -> Self {
        (lit.id, lit.sign)
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
