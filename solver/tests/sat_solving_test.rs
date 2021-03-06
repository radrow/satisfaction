use std::path::PathBuf;

use proptest::{bool::weighted, collection::vec, prelude::*};

use solver::{
    CadicalSolver, CNF, CNFClause, CNFVar, DLCS, DLIS,
    JeroslawWang, MOM, NaiveBranching, SatisfactionSolver, SATSolution, Solver,
};
use solver::cdcl::{
    CDCLSolver,
    branching_strategies::VSIDS,
    learning_schemes::RelSAT,
    restart_policies::{
        RestartFixed,
        RestartGeom,
        RestartLuby,
        RestartNever,
    },
    deletion_strategies::{NoDeletion, BerkMin},
    preprocessors::{NiVER, RemoveTautology, ListPreprocessor, NoPreprocessing},
};

const MAX_NUM_VARIABLES: usize = 50;
const MAX_NUM_LITERALS: usize = 10;
const MAX_NUM_CLAUSES: usize = 50;

fn setup_custom_solver() -> Vec<(&'static str, Box<dyn Solver>)> {
    let mut solvers: Vec<(&'static str, Box<dyn Solver>)> = Vec::new();
    //solvers.push(("CDCL-BerkMin-Never", Box::new(CDCLSolver::new(VSIDS, RelSAT, BerkMin::default(), RestartNever, NiVER))));
    //solvers.push(("CDCL-No-Never", Box::new(CDCLSolver::new(VSIDS, RelSAT, NoDeletion, RestartNever, RemoveTautology))));
    //solvers.push(("CDCL-Fixed", Box::new(CDCLSolver::new(VSIDS, RelSAT, NoDeletion, RestartFixed::default(), NoPreprocessing))));
    solvers.push(("CDCL-Fixed", Box::new(CDCLSolver::new(VSIDS, RelSAT, NoDeletion, RestartNever, NoPreprocessing, None))));
    //solvers.push(("CDCL-Geom", Box::new(CDCLSolver::new(VSIDS, RelSAT, NoDeletion, RestartGeom::default(), NoPreprocessing))));
    //solvers.push(("CDCL-Luby", Box::new(CDCLSolver::new(VSIDS, RelSAT, NoDeletion, RestartLuby::default(), NoPreprocessing)))); 

    //solvers.push(("NaiveBranching", Box::new(SatisfactionSolver::new(NaiveBranching))));
    //solvers.push(("JeroslawWang", Box::new(SatisfactionSolver::new(JeroslawWang))));
    //solvers.push(("DLIS", Box::new(SatisfactionSolver::new(DLIS))));
    //solvers.push(("DLCS", Box::new(SatisfactionSolver::new(DLCS))));
    //solvers.push(("MOM", Box::new(SatisfactionSolver::new(MOM))));
    solvers
}

fn solve_custom_solvers(formula: &CNF, solvers: Vec<(&'static str, Box<dyn Solver>)>) -> SATSolution {
    let mut solutions: Vec<SATSolution> = solvers.iter()
        .map(|(name, solver)| {
            println!("\nTesting {}\n", *name);
            let solution = solver.solve(formula);
            match solution.clone() {
                SATSolution::Satisfiable(assignment) => assert!(is_satisfied(formula.clauses.iter().cloned(), assignment), *name),
                _ => {},
            }
            solution
        }).collect();


    let result = solutions.iter()
        .zip(solvers.iter().map(|(name, _)| name))
        .skip(1)
        .try_fold(
            solutions[0].is_sat(),
            |acc, (solution, name)| {
                if solution.is_sat() == acc { Ok(acc) }
                else { Err(*name) }
            });

    match result {
        Ok(_) => {},
        Err(name) => panic!(format!("Branching strategy {} differs in satisfiability", name)),
    };

    solutions.pop().unwrap()
}

fn execute_solvers(formula: &CNF) -> (SATSolution, SATSolution) {
    println!("{:?}", &formula);

    let testing_solvers = setup_custom_solver();
    let reference_solver = CadicalSolver;

    let testing_solution = solve_custom_solvers(formula, testing_solvers);
    let reference_solution = reference_solver.solve(formula);

    (testing_solution, reference_solution)
}

fn is_satisfied(mut formula: impl Iterator<Item = CNFClause>, assignment: Vec<bool>) -> bool {
    formula.all(|clause| {
        clause.vars.iter().any(|var|
                // If sign is negative assignment is inverted
                // else it is passed by.
                !(assignment[var.id-1] ^ var.sign))
    })
}

#[test]
fn prescribed_instances() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/prescribed_instances");

    let process = |files: PathBuf, satisfiable: bool| {
        files
            .read_dir()
            .unwrap()
            .filter_map(|file| {
                let path = file.ok()?.path();

                if path.extension()? == "cnf" {
                    Some(path)
                } else {
                    None
                }
            })
            .for_each(|file| {
                println!("{:?}", file.file_stem().unwrap());
                let content = std::fs::read_to_string(file).unwrap();
                let formula = CNF::from_dimacs(&content).unwrap();

                let solvers = setup_custom_solver();
                assert!(
                    match solve_custom_solvers(&formula, solvers) {
                        SATSolution::Satisfiable(_) => true,
                        SATSolution::Unsatisfiable => false,
                        SATSolution::Unknown => panic!("Could not solve puzzle in time"),
                    } == satisfiable
                )
            })
    };

    process(path.join("sat"), true);
    process(path.join("unsat"), false);
}

#[test]
fn failed_proptest_instance() {
    let formula = CNF::from_dimacs(
r"
p cnf 3 1
2 -1 3
").unwrap();
    let (custom, reference) = execute_solvers(&formula);

    assert_eq!(custom.is_sat(), reference.is_sat());
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
