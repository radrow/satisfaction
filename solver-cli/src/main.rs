mod config;

use solver::SATSolution::Satisfiable;
use std::process::exit;
use clap::{App, Arg};
use config::Config;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use solver::{
    Solver,
    CNF,
    CadicalSolver,
    Bruteforce,
    SatisfactionSolver,
};

fn make_config<'a>() -> Config {
    let matches = App::new("satisfaction")
        .version("1.0")
        .author("Alex&Korbi&Radek inc.")
        .about("A tool to satisfy all your desires (or prove they are impossible)")
        .arg(Arg::with_name("input")
             .short("i")
             .long("input")
             .takes_value(true)
             .help("Input file"))
        .arg(Arg::with_name("algorithm")
             .long("algorithm")
             .value_name("ALGORITHM")
             .help("SAT solving algorithm")
             .takes_value(true)
             .possible_values(&["bruteforce", "cadical", "satisfaction"])
             .default_value("satisfaction"))
        .arg(Arg::with_name("branching")
             .long("branch")
             .help("DPLL branching strategy")
             .requires_if("algorithm", "satisfaction")
             .required_if("algorithm", "satisfaction")
             .possible_values(&["naive", "DLIS", "DLCS", "MOM", "AMOM", "Jeroslaw-Wang"]))
        .arg(Arg::with_name("return_code")
             .long("return-code")
             .short("r")
             .help("Will return 1 if satisfiable and 0 if not (useful for scripting)")
             .takes_value(false))
        .get_matches();

    let solver: Box<dyn Solver> = match matches.value_of("algorithm") {
        Some("bruteforce") => Box::new(Bruteforce::Bruteforce),
        Some("cadical") => Box::new(CadicalSolver),
        Some("satisfaction") =>
            match matches.value_of("branch") {
                Some("naive") => Box::new(SatisfactionSolver::new(solver::NaiveBranching)),
                Some("DLIS") => Box::new(SatisfactionSolver::new(solver::DLIS)),
                Some("DLCS") => Box::new(SatisfactionSolver::new(solver::DLCS)),
                Some("MOM") => Box::new(SatisfactionSolver::new(solver::MOM)),
                Some("AMOM") => Box::new(SatisfactionSolver::new(solver::ActiveMOM)),
                Some("Jeroslaw-Wang") => unimplemented!(),
                _ => unreachable!() // already handled by clap
            },
        _ => unreachable!() // already handled by clap
    };

    Config{
        input: matches.value_of("input").map(String::from),
        return_code: matches.is_present("return_code"),
        solver: Box::new(solver)
    }
}

fn get_input(handle: &mut dyn Read) -> io::Result<String> {
    let mut buffer = String::new();
    handle.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn solve_formula(solver: Box<dyn Solver>, formula: &CNF) -> solver::SATSolution {
    let solution = solver.solve(&formula);
    println!("{}", solution.to_dimacs());
    solution
}

fn main() {
    let config = make_config();

    let input = match config.input {
        None =>
            get_input(&mut io::stdin()),
        Some(file) =>
            get_input(&mut File::open(&file)
                      .expect(&("Couldn't open file ".to_string() + &file)))
    }.unwrap_or_else(|e| panic!(e));

    let formula = CNF::from_dimacs(&input).expect("Parse error");

    let solution = solve_formula(config.solver, &formula);

    if config.return_code {
        match solution {
            Satisfiable(_) => exit(1),
            _ => ()
        }
    }
}
