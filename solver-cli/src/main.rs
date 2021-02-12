mod config;

use clap::{App, Arg};
use config::Config;
use solver::SATSolution::{Satisfiable, Unknown};
use solver::{Bruteforce, CadicalSolver, SatisfactionSolver, Solver, CNF};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::exit;

fn make_config<'a>() -> Config {
    let matches = App::new("satisfaction")
        .version("1.0")
        .author("Alex&Korbi&Radek inc.")
        .about("A tool to satisfy all your desires (or prove they are impossible)")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .takes_value(true)
                .help("Input file"),
        )
        .arg(
            Arg::with_name("algorithm")
                .long("algorithm")
                .value_name("ALGORITHM")
                .help("SAT solving algorithm")
                .takes_value(true)
                .possible_values(&["bruteforce", "cadical", "satisfaction"])
                .default_value("satisfaction"),
        )
        .arg(
            Arg::with_name("branch")
                .long("branch")
                .help("DPLL branching strategy")
                .requires_if("algorithm", "satisfaction")
                .possible_values(&["naive", "DLIS", "DLCS", "MOM", "Jeroslaw-Wang"])
                .default_value("DLCS"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .help("File name for output in DIMACS format"),
        )
        .arg(
            Arg::with_name("return_code")
                .long("return-code")
                .short("r")
                .help("Will return 1 if satisfiable and 0 if not (useful for scripting)")
                .takes_value(false),
        )
        .get_matches();

    let solver: Box<dyn Solver> = match matches.value_of("algorithm") {
        Some("bruteforce") => Box::new(Bruteforce::Bruteforce),
        Some("cadical") => Box::new(CadicalSolver),
        Some("satisfaction") => match matches.value_of("branch") {
            Some("naive") => Box::new(SatisfactionSolver::new(solver::NaiveBranching)),
            Some("DLIS") => Box::new(SatisfactionSolver::new(solver::DLIS)),
            Some("DLCS") => Box::new(SatisfactionSolver::new(solver::DLCS)),
            Some("AMOM") => Box::new(SatisfactionSolver::new(solver::MOM)),
            Some("Jeroslaw-Wang") => Box::new(SatisfactionSolver::new(solver::JeroslawWang)),
            _ => unreachable!(), // already handled by clap
        },
        _ => unreachable!(), // already handled by clap
    };

    Config {
        input: matches.value_of("input").map(String::from),
        return_code: matches.is_present("return_code"),
        solver,
        output: matches.value_of("output").map(|file| PathBuf::from(file)),
    }
}

fn get_input(handle: &mut impl Read) -> io::Result<String> {
    let mut buffer = String::new();
    handle.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = make_config();

    let input = match config.input {
        None => {
            println!("No input file specified. Reading from standard input...");
            get_input(&mut io::stdin())
        }
        Some(file) => get_input(&mut File::open(&file)?),
    }?;

    let formula = CNF::from_dimacs(&input)?;
    let solution = config.solver.solve(&formula);

    match config.output {
        Some(path) => std::fs::write(path, solution.to_dimacs())?,
        None => println!("{}", solution.to_dimacs()),
    }

    Ok(if config.return_code {
        match solution {
            Satisfiable(_) => exit(1),
            _ => (),
        }
    } else {
        match solution {
            Unknown => exit(2),
            _ => (),
        }
    })
}
