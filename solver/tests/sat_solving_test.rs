use proptest::{
    prelude::*,
    collection::vec,
    bool::weighted,
};
use solver::{CadicalSolver, Solver, CNFClause, CNFVar, Assignment, CNF, SatisfactionSolver};

const MAX_NUM_VARIABLES: usize = 50;
const MAX_NUM_LITERALS: usize = 10;
const MAX_NUM_CLAUSES: usize = 5;

fn setup_custom_solver() -> SatisfactionSolver {
    SatisfactionSolver
}

fn execute_solvers(formula: CNF, num_variables: usize) -> (Assignment, Assignment) {
    println!("{:?}", &formula);

    let testing_solver = setup_custom_solver();
    let reference_solver = CadicalSolver;

    let testing_solution = testing_solver.solve(formula.clone(), num_variables);
    let reference_solution = reference_solver.solve(formula, num_variables);

    (testing_solution, reference_solution)
}


fn is_satisfied(mut formula: impl Iterator<Item=CNFClause>, assignment: Vec<bool>) -> bool {
    formula.all(|clause|
        clause.vars.iter()
            .any(|var|
                // If sign is negative assignment is inverted
                // else it is passed by.
                !(assignment[var.id-1] ^ var.sign)
            )
        )
}
    
proptest! {
    #[test]
    fn only_positive_unit_clauses(num_variables in 1..=MAX_NUM_VARIABLES ) {
        let formula = (1..=num_variables)
            .map(|variable| CNFClause::single(CNFVar{id: variable, sign: true}))
            .collect::<Vec<_>>();

        let (custom, reference) = execute_solvers(CNF{clauses:formula}, num_variables);

        prop_assert_eq!(custom, reference);
    }
    
    #[test]
    fn only_negative_unit_clauses(num_variables in 1..=MAX_NUM_VARIABLES ) {
        let formula = (1..=num_variables)
            .map(|variable| CNFClause::single(CNFVar{id: variable, sign: false}))
            .collect::<Vec<_>>();

        let (custom, reference) = execute_solvers(CNF{clauses:formula}, num_variables);

        prop_assert_eq!(custom, reference);
    }

    #[test]
    fn only_unit_clauses(signs in vec(weighted(0.5), 1..=MAX_NUM_VARIABLES)) {
        let num_variables = signs.len() ;
        let formula = signs.iter()
            .cloned()
            .enumerate()
            .map(|(id, sign)| {
                CNFClause::single(CNFVar{id: id+1, sign})
            }).collect::<Vec<_>>();

        let (custom, reference) = execute_solvers(CNF{clauses:formula}, num_variables);
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

        let formula = clauses.iter()
            .map(|clause| 
                CNFClause{ vars:
                clause.iter()
                    .cloned()
                    .map(|(id, sign)| {
                        CNFVar{id, sign}
                    }).collect()
                }).collect::<Vec<_>>();

        let (custom, reference) = execute_solvers(CNF{clauses:formula.clone()}, num_variables);

        // The result regarding satisfiability is correct.
        prop_assert_eq!(custom.is_unsat(), reference.is_unsat());
        prop_assert_eq!(custom.is_unknown(), reference.is_unknown());

        // The found assignment does indeed satisfies the formula.
        if let Assignment::Satisfiable(assignment) = custom {
            println!("{:?}", assignment);
            prop_assert!(is_satisfied(formula.into_iter(), assignment));
        }
    }
}
