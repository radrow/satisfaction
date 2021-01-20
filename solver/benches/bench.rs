extern crate solver;

use criterion::{black_box, criterion_group, criterion_main, Criterion, BatchSize};
use solver::{SATSolution, SatisfactionSolver, CadicalSolver, NaiveBranching, CNF, Solver, BranchingStrategy};
use std::path::{PathBuf, Path};
use std::time::Duration;


fn load_formulae(directory: impl AsRef<Path>) -> impl Iterator<Item=(String, CNF)> {
    directory.as_ref().read_dir()
        .unwrap()
        .filter_map(|file| {
            let path = file.ok()?
                .path();

            if path.extension()? == "cnf" { Some(path) }
            else { None }
        }).map(|file| {
            let content = std::fs::read_to_string(&file).unwrap();
            let formula = CNF::from_dimacs(&content).unwrap();
            (file.file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned(),
                formula)
        })
}

fn create_group_for_solver(c: &mut Criterion, name: &str, strategy: impl Solver, path: impl AsRef<Path>) {
    let mut group = c.benchmark_group(name);
    let solver = std::rc::Rc::new(strategy);

    for (name, formula) in load_formulae(path) {
        let solver = solver.clone();
        group.bench_function(name, move |b| {
            b.iter_batched(|| formula.clone(),|formula| solver.solve(formula), BatchSize::SmallInput)
        });
    }

    group.finish()
}

fn criterion_benchmark(c: &mut Criterion) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches/inputs");

    // Change input directory
    let sat_dir = path.join("sat");
    // Append branching strategy
    create_group_for_solver(c, "Naive Branching", SatisfactionSolver::new(NaiveBranching), &sat_dir);
    create_group_for_solver(c, "Cadical", CadicalSolver, &sat_dir);
}

criterion_group!{
    name = benches;
    config = Criterion::default();
    targets = criterion_benchmark
}
criterion_main!(benches);
