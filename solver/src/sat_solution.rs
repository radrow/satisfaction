pub type Valuation = Vec<bool>;

#[derive(PartialEq, Eq)]
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
            SATSolution::Satisfiable(_)  => true,
            _                           => false,
        }
    }

    pub fn is_unsat(&self) -> bool {
        match self {
            SATSolution::Unsatisfiable   => true,
            _                           => false,
        }
    }

    pub fn is_unknown(&self) -> bool {
        match self {
            SATSolution::Unknown => true,
            _                   => false,
        }
    }

    pub fn to_dimacs(&self) -> String {
        format!("s {}\n",
            match self {
                SATSolution::Unsatisfiable => "UNSATISFIABLE".to_string(),
                SATSolution::Unknown => "UNKNOWN".to_string(),
                SATSolution::Satisfiable(variables) => {
                    format!("SATISFIABLE\nv {} 0",
                        variables.iter()
                            .enumerate()
                            .map(|(id, sign)| {
                                format!("{}{}",
                                    if *sign { " " }
                                    else { "-" },
                                    id+1)
                            }).collect::<Vec<String>>().join(" ")
                        )
                }
            })
    }
}

impl std::fmt::Debug for SATSolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.to_dimacs())
    }
}
