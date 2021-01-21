use proptest::{
    prelude::*,
    collection::vec,
    bool::weighted,
};
use std::path::PathBuf;
use solver::{CadicalSolver, Solver, CNFClause, CNFVar, SATSolution, CNF, SatisfactionSolver, NaiveBranching, JeroslawWang, DLCS, DLIS, MOM};

const MAX_NUM_VARIABLES: usize = 50;
const MAX_NUM_LITERALS: usize = 10;
const MAX_NUM_CLAUSES: usize = 50;

fn setup_custom_solver() -> SatisfactionSolver<NaiveBranching> {
    SatisfactionSolver::new(NaiveBranching)
}

fn execute_solvers(formula: &CNF) -> (SATSolution, SATSolution) {
    println!("{:?}", &formula);

    let testing_solver = setup_custom_solver();
    let reference_solver = CadicalSolver;

    let testing_solution = testing_solver.solve(formula);
    let reference_solution = reference_solver.solve(formula);

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

#[test]
fn prescribed_instances() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/prescribed_instances");

    let solver = SatisfactionSolver::new(MOM);
    //let solver = SatisfactionSolver::new(DLIS);
    //let solver = SatisfactionSolver::new(DLCS);

    let process = |files: PathBuf, satisfiable: bool| {
        files.read_dir()
            .unwrap()
            .filter_map(|file| {
                let path = file.ok()?
                    .path();

                if path.extension()? == "cnf" { Some(path) }
                else { None }
            }).for_each(|file| {
                println!("{:?}", file.file_stem().unwrap());
                let content = std::fs::read_to_string(file).unwrap();
                let formula = CNF::from_dimacs(&content).unwrap();

                assert!(match solver.solve(&formula) {
                    SATSolution::Satisfiable(_) => true,
                    SATSolution::Unsatisfiable => false,
                    SATSolution::Unknown => unreachable!(),
                } == satisfiable)
            })
    };

    process(path.join("sat"), true);
    process(path.join("unsat"), false);
}


#[test]
fn failed_proptest_instance() {
    let formula = CNF { 
        clauses: vec![
            CNFClause {
                vars: vec![
                    CNFVar::new(37, false),
                    CNFVar::new(39, false),
                ]
            },
            CNFClause {
                vars: vec![
                    CNFVar::new(37, false),
                    CNFVar::new(39, true),
                ]
            },
        ],
        num_variables: 39
    };
    let (custom, reference) = execute_solvers(&formula);

    assert_eq!(custom, reference);
}


proptest! {
    #[test]
    fn only_positive_unit_clauses(num_variables in 1..=MAX_NUM_VARIABLES) {
        let formula = (1..=num_variables)
            .map(|variable| CNFClause::single(CNFVar{id: variable, sign: true}))
            .collect::<Vec<_>>();

        let (custom, reference) = execute_solvers(&CNF { clauses: formula, num_variables });

        prop_assert_eq!(custom, reference);
    }

    #[test]
    fn only_negative_unit_clauses(num_variables in 1..=MAX_NUM_VARIABLES ) {
        let formula = (1..=num_variables)
            .map(|variable| CNFClause::single(CNFVar{id: variable, sign: false}))
            .collect::<Vec<_>>();

        let (custom, reference) = execute_solvers(&CNF { clauses: formula, num_variables });

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

        let (custom, reference) = execute_solvers(&CNF { clauses: formula, num_variables });
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

        let (custom, reference) = execute_solvers(&CNF { clauses:formula.clone(), num_variables });

        // The result regarding satisfiability is correct.
        prop_assert_eq!(custom.is_unsat(), reference.is_unsat());
        prop_assert_eq!(custom.is_unknown(), reference.is_unknown());

        // The found assignment does indeed satisfies the formula.
        if let SATSolution::Satisfiable(assignment) = custom {
            println!("{:?}", assignment);
            prop_assert!(is_satisfied(formula.into_iter(), assignment));
        }
    }
}
