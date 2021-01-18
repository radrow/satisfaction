// pub struct Assignment(Option<Vec<bool>>);

#[derive(PartialEq, Eq)]
pub enum Assignment {
    Satisfiable(Vec<bool>),
    Unsatisfiable,
    Unknown,
}

impl std::iter::FromIterator<bool> for Assignment {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        Assignment::Satisfiable(iter.into_iter().collect())
    }
}

impl Assignment {
    pub fn is_sat(&self) -> bool {
        match self {
            Assignment::Satisfiable(_)  => true,
            _                           => false,
        }
    }

    pub fn is_unsat(&self) -> bool {
        match self {
            Assignment::Unsatisfiable   => true,
            _                           => false,
        }
    }

    pub fn is_unknown(&self) -> bool {
        match self {
            Assignment::Unknown => true,
            _                   => false,
        }
    }

    pub fn to_dimacs(&self) -> String {
        format!("s {}\n",
            match self {
                Assignment::Unsatisfiable => "UNSATISFIABLE".to_string(),
                Assignment::Unknown => "UNKNOWN".to_string(),
                Assignment::Satisfiable(variables) => {
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

impl std::fmt::Debug for Assignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.to_dimacs())
    }
}
