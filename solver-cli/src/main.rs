mod cnf;

use cnf::CNF;
use cnf::CNFClause;
use cnf::CNFVar;
use std::fmt;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
enum VarValue {
    Pos, Neg, Free
}

struct Variable {
    value: VarValue, 
    pos_occ: Vec<usize>,
    neg_occ: Vec<usize>
}

struct Clause {
    active_cl: usize,
    satisfied: bool,
    literals: Vec<isize>
}

type Variables = Vec<Variable>;
type Clauses = Vec<Clause>;

impl Variable {
    fn new(cnf: &CNF, var_num: usize) -> Variable {
        Variable {
            value: VarValue::Free,
            neg_occ: cnf.clauses.iter().enumerate().filter_map(|(index, clause)| {
                    if clause.vars.contains(&CNFVar::Neg(var_num as u32)) {
                        Some(index)
                    } else {
                        None
                    }
                }).collect(),
            pos_occ: cnf.clauses.iter().enumerate().filter_map(|(index, clause)| {
                    if clause.vars.contains(&CNFVar::Pos(var_num as u32)) {
                        Some(index)
                    } else {
                        None
                    }
                }).collect()
        }
    }
}

impl Clause {
    fn new(cnf_clause: &CNFClause) -> Clause {
        Clause {
            active_cl: cnf_clause.vars.len(),
            satisfied: false,
            literals: cnf_clause.vars.iter().map(|var| {
                match var {
                    CNFVar::Pos(s) => *s as isize,
                    CNFVar::Neg(s) => -(*s as isize)
                }
            }).collect()
        }
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "act: {}, sat: {}, lit: ", self.active_cl, self.satisfied);
        self.literals.iter().for_each(|lit| {write!(f, "{} ", lit);});
        write!(f, "\n")
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "val: {:?}, pos: ", self.value);
        self.pos_occ.iter().for_each(|var| {write!(f, "{} ", var);});
        write!(f, "| neg: ");
        self.neg_occ.iter().for_each(|var| {write!(f, "{} ", var);});
        write!(f, "\n")
    }
}

fn create_data_structures(cnf: &CNF, num_of_vars: usize) -> (Variables, Clauses) {
    let mut variables: Variables = Vec::new();
    let mut clauses: Clauses = Vec::new();

    cnf.clauses.iter().for_each(|cnf_clause| clauses.push(Clause::new(cnf_clause)));
    (1..=num_of_vars).for_each(|i| variables.push(Variable::new(cnf, i)));

    (variables, clauses)
}

fn dpll(cnf: &CNF, num_of_vars: usize) {
    let (mut variables, mut clauses) = create_data_structures(&cnf, num_of_vars);
    let mut unit_queue: VecDeque<usize> = VecDeque::new();

    for mut var in variables {
        var.value = VarValue::Pos;

        var.pos_occ.iter().for_each(|p_occ| clauses[*p_occ].active_cl -= 1);
        var.neg_occ.iter().for_each(|n_occ| clauses[*n_occ].active_cl -= 1);

        for cl in &clauses {
            if cl.active_cl == 1 {
                
            }
        }
    }
}

fn main() {
    let input: Vec<Vec<isize>> = vec![vec![-1, 2], vec![-1, 3], vec![-2, 4], vec![-3, -4, 5], vec![-4, -5, 6, 7], vec![4, 5, -6]];
    let mut cnf: CNF = CNF::new();

    for clause in input {
        let mut cnf_clause: CNFClause = CNFClause::new();
        for var in clause {
            if var > 0 {
                cnf_clause.push(CNFVar::Pos(var.abs() as u32));
            } else {
                cnf_clause.push(CNFVar::Neg(var.abs() as u32));
            }
        }
        cnf.push(cnf_clause);
    }
    let (variables, clauses) = create_data_structures(&cnf, 7);
    for var in variables {
        print!("{}", var);
    }
    for cl in clauses {
        print!("{}", cl);
    }

}
