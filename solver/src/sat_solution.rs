use std::fmt::Write as FmtWrite;

pub type Valuation = Vec<bool>;

const MAX_LITERALS_PER_LINE: usize = 8;

/// An enum which represents all output possibilities for solving a SAT-problem.
/// * `Satisfiable(Valuation) - The an satisfying assignment was found.
///     `Valuation` a Vec of booleans representing a contiguous range from variable with ID 1
///     (index 0) to the variable with the maximal ID).
/// * `Unsatisfiable` - The formula is not satisfiable i.e. to assignment makes the formula true.
/// * `Unknown` - A satisfying assignment could not be found, however, there still could be one.
#[derive(PartialEq, Eq, Clone)]
pub enum SATSolution {
    Satisfiable(Valuation),
    Unsatisfiable,
    Unknown,
}

impl std::iter::FromIterator<bool> for SATSolution {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        SATSolution::Satisfiable(iter.into_iter().collect())
    }
}

impl SATSolution {
    pub fn is_sat(&self) -> bool {
        match self {
            SATSolution::Satisfiable(_) => true,
            _ => false,
        }
    }

    pub fn is_unsat(&self) -> bool {
        match self {
            SATSolution::Unsatisfiable => true,
            _ => false,
        }
    }

    pub fn is_unknown(&self) -> bool {
        match self {
            SATSolution::Unknown => true,
            _ => false,
        }
    }

    /// Transform it a output
    pub fn to_dimacs(&self) -> String {
        format!(
            "s {}\n",
            match self {
                SATSolution::Unsatisfiable => "UNSATISFIABLE".to_string(),
                SATSolution::Unknown => "UNKNOWN".to_string(),
                SATSolution::Satisfiable(variables) => {
                    format!("SATISFIABLE\n{}", {
                        let mut out = String::new();
                        let mut iter = variables.iter().enumerate().peekable();

                        while iter.peek().is_some() {
                            out.push('v');
                            out.push(' ');
                            for (id, sign) in iter.by_ref().take(MAX_LITERALS_PER_LINE) {
                                write!(&mut out, "{}{}", if *sign { " " } else { "-" }, id + 1)
                                    .unwrap();
                                out.push(' ');
                            }
                            out.push('0');
                            out.push('\n');
                        }
                        out
                    })
                }
            }
        )
    }
}

impl std::fmt::Debug for SATSolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.to_dimacs())
    }
}

impl std::fmt::Display for SATSolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SATSolution::Unsatisfiable => "Unsatisfiable".to_string(),
                SATSolution::Unknown => "Unknown".to_string(),
                SATSolution::Satisfiable(variables) => {
                    format!("Satisfiable:\n{}", {
                        let mut out = String::new();
                        let mut iter = variables.iter().enumerate().peekable();

                        while iter.peek().is_some() {
                            for (id, sign) in iter.by_ref().take(MAX_LITERALS_PER_LINE) {
                                write!(&mut out, "{}{}", if *sign { " " } else { "-" }, id + 1)?;
                                out.push(' ');
                            }
                            out.push('\n');
                        }
                        out
                    })
                }
            }
        )
    }
}
