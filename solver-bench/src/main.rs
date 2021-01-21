mod config;
mod plotting;

use std::{
    path::{Path, PathBuf},
    time::Duration,
    io,
    io::prelude::*,
    collections::HashMap,
    fs::File,
};
use clap::{App, Arg};
use config::Config;
use plotting::plot_runtimes;
use solver::{
    TimedSolver,
    TimeLimitedSolver,
    SATSolution,
    CNF,
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
        .arg(Arg::with_name("output")
            .short("o")
            .long("output")
            .takes_value(true)
            .required(false)
            .help("Output file for plot"))
        .arg(Arg::with_name("time")
            .short("t")
            .long("time")
            .takes_value(true)
            .required(false)
            .default_value("60")
            .help("Timeout for a single instance in seconds"))
        .arg(Arg::with_name("return_code")
             .long("return-code")
             .short("r")
             .help("Will return 1 if satisfiable and 0 if not (useful for scripting)")
             .takes_value(false))
        .get_matches();

    let solvers : Vec<(String, Box<dyn TimeLimitedSolver>)> =
        vec![
            // Brute to expensive
            ("DLIS".to_string(),  Box::new(SatisfactionSolver::new(solver::DLIS))),
            ("DLCS".to_string(),  Box::new(SatisfactionSolver::new(solver::DLCS))),
            ("MOM".to_string(),  Box::new(SatisfactionSolver::new(solver::MOM))),
            ("Jeroslaw-Wang".to_string(),  Box::new(SatisfactionSolver::new(solver::JeroslawWang))),
        ];

    Config{
        input: matches.value_of("input").map(String::from).unwrap(),
        plot: matches.is_present("plot"),
        output: PathBuf::from(matches.value_of("output").unwrap_or("out.svg")),
        solvers,
        max_duration: matches.value_of("time").map(|t| t.parse::<u64>().expect("Time must be a number")).unwrap_or(60),
    }
}

fn load_files(dir: &Path) -> io::Result<Vec<CNF>> {
    let mut out = Vec::new();
    if dir.is_dir() {
        let dir_stream = std::fs::read_dir(dir)?;
        for entry in dir_stream {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let mut buffer = String::new();
                let mut handle = File::open(path)?;
                handle.read_to_string(&mut buffer)?;
                let formula = CNF::from_dimacs(&buffer).expect("Parse error");
                out.push(formula)
            }
        }
    }
    Ok(out)
}

fn run_tests<S: TimeLimitedSolver>(formulae: &Vec<CNF>, solver: TimedSolver<S>, max_duration: Duration) -> Vec<Duration> {
    formulae.iter()
        .filter_map(|formula| {
            let (duration, solution) = solver.solve_timed(formula, max_duration);
            match solution {
                SATSolution::Unknown    => None,
                _                       => Some(duration),
            }
        }).collect()
}

/// Returns a vector of test results; for each solver duration on each test
fn run_benchmark(formulae: Vec<CNF>, solvers: Vec<(String, Box<dyn TimeLimitedSolver>)>, max_duration: Duration) -> HashMap<String, Vec<Duration>> {
    solvers.into_iter()
        .map(|(name, solver)| {
            let solver = TimedSolver::new(solver);
            println!("Started {}", &name);
            let result = run_tests(&formulae, solver, max_duration);
            println!("Finished {}", &name);
            (name, result)
        }).collect()
}

fn main() {
    let config = make_config();
    let test_formulae =
        load_files(Path::new(&config.input)).unwrap_or_else(|e| panic!(e));

    let benchmarks = run_benchmark(test_formulae, config.solvers, Duration::from_secs(config.max_duration));
    plot_runtimes(benchmarks, config.output, (600, 480)).unwrap();
}
