use std::fmt;
use std::collections::HashMap;

#[derive (Clone)]  // :(
pub struct CNF {
    pub clauses : Vec<CNFClause>
}

#[derive (Clone)]
pub struct CNFClause {
    pub vars : Vec<CNFVar>
}

#[derive (Clone)]
pub enum CNFVar {
    Pos(String),
    Neg(String)
}

impl CNF {
    pub fn cat(&self, c : &CNF) -> CNF {
        let mut clauses : Vec<CNFClause> = vec![];
        clauses.extend(self.clauses.clone());
        clauses.extend(c.clauses.clone());
        CNF {
            clauses: clauses
        }
    }

    pub fn create_variable_mapping(&self) -> HashMap<String, i32> {
        let mut var_map : HashMap<String, i32> = HashMap::new();
        let mut count : i32 = 0;

        for clause in &self.clauses {
            for var in &clause.vars {
                let name = match var {
                    CNFVar::Pos(s) => s,
                    CNFVar::Neg(s) => s
                };
                var_map.entry(name.clone())
                    .or_insert({count += 1; count});
            }
        }
        var_map
    }

    pub fn to_dimacs(&self) -> String {
        let mut out : String = String::from("");
        let var_map = self.create_variable_mapping();

        out.extend("p cnf ".chars());
        out.extend(var_map.len().to_string().chars());
        out.extend(" ".chars());
        out.extend(self.clauses.len().to_string().chars());
        out.extend("\n".chars());

        for clause in &self.clauses {
            for var in &clause.vars {
                let var: i32 = match var {
                    CNFVar::Pos(s) => *var_map.get(s).unwrap(),
                    CNFVar::Neg(s) => -*var_map.get(s).unwrap(),
                };

                out.extend(var.to_string().chars());
                out.extend(" ".chars())
            }
            out.extend("0\n".chars())
        }
        out
    }
}

impl CNFClause {
    pub fn cat(&self, c : &CNFClause) -> CNFClause {
        let mut vars : Vec<CNFVar>  = vec![];
        vars.extend(self.vars.clone());
        vars.extend(c.vars.clone());
        CNFClause {
            vars: vars
        }
    }
}


impl fmt::Display for CNF {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in &self.clauses {
            write!(f, "{}\n", c);
        }
        write!(f, "")
    }
}
impl fmt::Display for CNFClause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in &self.vars {
            write!(f, "({})  ", c);
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
