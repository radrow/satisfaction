mod config;

use clap::{App, Arg};
use config::Config;
use solver::bruteforce::*;
use solver::timed_solver::*;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use solver::{sat_solver::Solver, cnf::CNF, SATSolution};

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
        .arg(Arg::with_name("plot")
             .long("plot")
             .short("p")
             .help("Plot performance benchmark")
             .takes_value(false))
        .arg(Arg::with_name("return_code")
             .long("return-code")
             .short("r")
             .help("Will return 1 if satisfiable and 0 if not (useful for scripting)")
             .takes_value(false))
        .get_matches();

    let solver = match matches.value_of("algorithm").unwrap() {
        "bruteforce" => Bruteforce::Bruteforce,
        "cadical" => panic!("Not supported"),
        "satisfaction" => panic!("Not supported"),
        _ => panic!("Unknown algorithm")
    };



    Config{
        input: matches.value_of("input").map(String::from),
        return_code: matches.is_present("return_code"),
        plot: matches.is_present("plot"),
        solver: Box::new(solver)
    }
}

fn get_input(handle: &mut dyn Read) -> io::Result<String> {
    let mut buffer = String::new();
    handle.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn solve_formula(solver: Box<dyn Solver>, formula: CNF) {
    match solver.solve(formula) {
        SATSolution::Unsatisfiable => println!("nah"),
        SATSolution::Satisfiable(solution) => println!("Solved!\n{:?}", solution),
        SATSolution::Unknown => unreachable!()
    }
}

fn solve_plot_formula(solver: Box<dyn Solver>, formula: CNF) {
    let solver_t = TimedSolver::new(solver);
    let (_duration, _solution) = solver_t.solve_timed(formula);
}

fn main() {
    let config = make_config();

    let input = match config.input {
        None => get_input(&mut io::stdin()),
        Some(file) => get_input(&mut File::open(&file).expect(&("Couldn't open file ".to_string() + &file)))
    }.unwrap_or_else(|e| panic!(e));

    let formula = CNF::from_dimacs(&input).expect("Parse error");

    if config.plot {
        solve_plot_formula(config.solver, formula)
    } else {
        solve_formula(config.solver, formula)
    }
}
