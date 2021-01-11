use proptest::{
    prelude::*,
    collection::vec,
    bool::weighted,
};
use solver::{CadicalSolver, Solver, CNFClause, CNFVar, Assignment, SatisfactionSolver};

const MAX_NUM_VARIABLES: usize = 3;
const MAX_NUM_LITERALS: usize = 3;
const MAX_NUM_CLAUSES: usize = 5;

fn execute_solvers(formula: Vec<CNFClause>, num_variables: usize) -> (Option<Assignment>, Option<Assignment>) {
    let testing_solver = SatisfactionSolver; // TODO: Replace testing_solver with custom solver
    let reference_solver = CadicalSolver;

    let testing_solution = testing_solver.solve(formula.clone().into_iter(), num_variables);
    let reference_solution = reference_solver.solve(formula.into_iter(), num_variables);

    (testing_solution, reference_solution)
}
    
proptest! {
    #[test]
    fn only_positive_unit_clauses(num_variables in 1..=MAX_NUM_VARIABLES ) {
        let formula = (1..=num_variables)
            .map(|variable| CNFClause{vars: vec![CNFVar::Pos(variable)]})
            .collect();

        let (custom, reference) = execute_solvers(formula, num_variables);

        prop_assert_eq!(custom, reference);
    }
    
    #[test]
    fn only_negative_unit_clauses(num_variables in 1..=MAX_NUM_VARIABLES ) {
        let formula = (1..=num_variables)
            .map(|variable| CNFClause{vars: vec![CNFVar::Neg(variable)]})
            .collect();

        let (custom, reference) = execute_solvers(formula, num_variables);

        prop_assert_eq!(custom, reference);
    }

    #[test]
    fn only_unit_clauses(signs in vec(weighted(0.5), 1..=MAX_NUM_VARIABLES)) {
        let num_variables = signs.len() ;
        let formula = signs.iter()
            .enumerate()
            .map(|(variable, sign)| {
                let literal = match sign {
                    true => CNFVar::Pos,
                    false => CNFVar::Neg,
                };
                CNFClause{vars: vec![literal((variable )+1)]}
            }).collect();

        let (custom, reference) = execute_solvers(formula, num_variables);
        prop_assert_eq!(custom, reference);
    }

    #[test]
    fn arbitrary_cnf_formula(clauses in vec(vec((1..=MAX_NUM_VARIABLES, weighted(0.5)), 1..=MAX_NUM_LITERALS), 1..=MAX_NUM_CLAUSES)) {
        let num_variables = clauses.iter()
            .max_by_key(|clause| clause.len())
            .expect("Illegally, there were zero clauses!").len() ;

        let formula = clauses.iter()
            .map(|clause| 
                CNFClause{ vars:
                clause.iter()
                    .map(|(variable, sign)| {
                        let literal = match sign {
                            true => CNFVar::Pos,
                            false => CNFVar::Neg,
                        };
                        literal(*variable )
                    }).collect()
                }
            ).collect();

        let (custom, reference) = execute_solvers(formula, num_variables);
        prop_assert_eq!(custom, reference);
    }
}
