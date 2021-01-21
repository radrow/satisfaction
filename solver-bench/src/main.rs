mod config;

use std::path::Path;
use std::time::Duration;
use clap::{App, Arg};
use config::Config;
use solver::timed_solver::*;
use solver::time_limited_solver::*;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap;
use solver::{
    SATSolution
    Solver,
    CNF,
    CadicalSolver,
    Bruteforce,
    SatisfactionSolver,
};

fn make_config<'a>() -> Config {
    let matches = App::new("satisfaction benchmarking")
        .version("1.0")
        .author("Alex&Korbi&Radek inc.")
        .about("Racing pit for SAT solvers")
        .arg(Arg::with_name("input")
             .short("i")
             .long("input")
             .takes_value(true)
             .required(true)
             .help("Directory of testing cases"))
        .arg(Arg::with_name("return_code")
             .long("return-code")
             .short("r")
             .help("Will return 1 if satisfiable and 0 if not (useful for scripting)")
             .takes_value(false))
        .get_matches();

    let solvers : Vec<(String, Box<dyn Solver>)> =
        vec![
            // Brute to expensive
            // ("brute".to_string()  ,    Box::new(Bruteforce::Bruteforce)),
            ("cadical".to_string(),    Box::new(CadicalSolver)),
            ("dpll-naive".to_string(), Box::new(SatisfactionSolver::new(solver::NaiveBranching))),
            ("dpll-dlis".to_string(),  Box::new(SatisfactionSolver::new(solver::DLIS))),
            ("dpll-dlcs".to_string(),  Box::new(SatisfactionSolver::new(solver::DLCS))),
            ("dpll-mom".to_string(),   Box::new(SatisfactionSolver::new(solver::MOM))),
            ("dpll-amom".to_string(),  Box::new(SatisfactionSolver::new(solver::ActiveMOM))),
        ];

    Config{
        input: matches.value_of("input").map(String::from).unwrap(),
        plot: matches.is_present("plot"),
        solvers,
    }
}

fn load_files(dir: &Path) -> io::Result<Vec<CNF>> {
    let mut out = Vec::new();
    if dir.is_dir() {
        let dir_stream = std::fs::read_dir(dir)?;
        for entry in dir_stream {
            let entry = entry?;
            let path = entry.path();
            let filename = entry.file_name();
            if !path.is_file() {
                let mut buffer = String::new();
                let mut handle = File::open(path)?;
                handle.read_to_string(&mut buffer)?;
                let formula = CNF::from_dimacs(&buffer).expect("Parse error");
                let testfile = match filename.to_str() {
                    None => "???",
                    Some(n) => n
                };
                out.push(formula)
            }
        }
    }
    Ok(out)
}

fn run_tests<S: Solver>(formulae: &Vec<CNF>, solver: TimeLimitedSolver<TimedSolver<S>>) -> Vec<Duration> {
    formulae.iter()
        .filter_map(|formula| {
            let (duration, solution) = solver.solve_timed(formula);
            match solution {
                SATSolution::Unknown    => None,
                _                       => Some(duration),
            }
        }).collect()
}

/// Returns a vector of test results; for each solver duration on each test
fn run_benchmark(formulae: Vec<CNF>, solvers: Vec<(String, Box<dyn Solver>)>, max_duration: Duration) -> HashMap<String, Vec<Duration>> {
    solvers.into_iter()
        .map(|(name, solver)| {
            let solver = TimedSolver::new(solver);
            let solver = TimeLimitedSolver::new(solver, max_duration);
            (name, run_tests(&formulae, solver))
        }).collect()
}

fn main() {
    let config = make_config();
    let max_duration = Duration::from_secs(60);

    let test_formulae =
        load_files(Path::new(&config.input)).unwrap_or_else(|e| panic!(e));

    let benchmarks = run_benchmark(test_formulae, config.solvers, max_duration);
}
