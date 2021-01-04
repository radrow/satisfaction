use proptest::{
    prelude::*,
    collection::vec,
    bool::weighted,
};
use solver::{CadicalSolver, Solver, CNFClause, CNFVar, Assignment};

const MAX_NUM_VARIABLES: usize = 50;
const MAX_NUM_LITERALS: usize = 10;
const MAX_NUM_CLAUSES: usize = 5;

fn setup_custom_solver() -> CadicalSolver {
    // TODO: Replace with custom solver
    CadicalSolver
}

fn execute_solvers(formula: Vec<CNFClause>, num_variables: usize) -> (Option<Assignment>, Option<Assignment>) {
    let testing_solver = setup_custom_solver();
    let reference_solver = CadicalSolver;

    let testing_solution = testing_solver.solve(formula.iter().cloned(), num_variables);
    let reference_solution = reference_solver.solve(formula.into_iter(), num_variables);

    (testing_solution, reference_solution)
}


fn is_satisfied(mut formula: impl Iterator<Item=CNFClause>, assignment: Assignment) -> bool {
    formula.all(|clause|
            clause.vars
                .iter()
                .any(|v| match v {
                    CNFVar::Pos(s) => assignment[*s-1],
                    CNFVar::Neg(s) => !assignment[*s-1],
                }))
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
            .map(|clause| clause.iter()
                .map(|(var, _)| *var)
                .max()
                .expect("There are empty clauses!"))
            .max()
            .expect("There are zero clauses!");

        let formula: Vec<CNFClause> = clauses.iter()
            .map(|clause| 
                CNFClause{ vars:
                clause.iter()
                    .map(|(variable, sign)| {
                        let literal = match sign {
                            true => CNFVar::Pos,
                            false => CNFVar::Neg,
                        };
                        literal(*variable)
                    }).collect()
                }).collect();

        println!("{:?}", &formula);
        let (custom, reference) = execute_solvers(formula.clone(), num_variables);

        // The result regarding satisfiability is correct.
        prop_assert_eq!(custom.is_none(), reference.is_none());

        // The found assignment does indeed satisfies the formula.
        if let Some(assignment) = custom {
            println!("{:?}", assignment);
            prop_assert!(is_satisfied(formula.into_iter(), assignment));
        }
    }
}
