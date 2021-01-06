mod dpll;
mod cnf;

use cnf::CNF;
use cnf::CNFClause;
use cnf::CNFVar;
use dpll::dpll;

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
    match dpll(&cnf, 7) {
        Ok(variables) => variables.iter().for_each(|v| print!("{}", v)),
        Err(e) => print!("{}", e)
    }
}
