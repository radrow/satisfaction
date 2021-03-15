mod config;

use clap::{App, Arg};
use config::Config;
use solver::{
    SATSolution::{
        Satisfiable,
        Unknown
    },
    cdcl::{
        branching_strategies::BranchingStrategyFactory,
        deletion_strategies::ClauseDeletionStrategyFactory,
        learning_schemes::{LearningSchemeFactory},
        preprocessors::{
            ListPreprocessor,
            NiVER,
            PreprocessorFactory,
            NoPreprocessing,
            RemoveTautology
        },
        restart_policies::RestartPolicyFactory
    }};
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
                .possible_values(&["bruteforce", "cadical", "dpll", "cdcl"])
                .default_value("cdcl"),
        )
        .arg(
            Arg::with_name("dpll-branching")
                .long("dpll-branching")
                .help("DPLL branching strategy")
                .requires_if("algorithm", "dpll")
                .possible_values(&["naive", "DLIS", "DLCS", "MOM", "Jeroslaw-Wang"])
                .default_value("DLCS"),
        )
        .arg(
            Arg::with_name("cdcl-restart")
                .long("cdcl-restart")
                .help("CDCL restarting policy")
                .requires_if("algorithm", "cdcl")
                .possible_values(&["fixed", "geom", "luby", "never"])
                .default_value("luby"),
        )
        .arg(
            Arg::with_name("cdcl-deletion")
                .long("cdcl-deletion")
                .help("CDCL clause deletion strategy")
                .requires_if("algorithm", "cdcl")
                .possible_values(&["never", "berk-min"])
                .default_value("berk-min"),
        )
        .arg(
            Arg::with_name("cdcl-branching")
                .long("cdcl-branching")
                .help("CDCL branching strategy")
                .requires_if("algorithm", "cdcl")
                .possible_values(&["VSIDS"])
                .default_value("VSIDS"),
        )
        .arg(
            Arg::with_name("cdcl-learning")
                .long("cdcl-learning")
                .help("CDCL learning schema")
                .requires_if("algorithm", "cdcl")
                .possible_values(&["relsat"])
                .default_value("relsat"),
        )
        .arg(
            Arg::with_name("cdcl-preproc")
                .long("cdcl-preproc")
                .help("CDCL preprocessing algorithms")
                .requires_if("algorithm", "cdcl")
                .possible_values(&["niver", "tautologies", "empty"])
                .default_value("empty"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .help("File name for output in DIMACS format"),
        )
        .arg(
            Arg::with_name("drup")
                .long("drup")
                .requires_if("algorithm", "cdcl")
                .takes_value(true)
                .help("DRUP proof output file. Currently only for CDCL.")
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
        Some("dpll") => match matches.value_of("dpll-branching") {
            Some("naive") => Box::new(SatisfactionSolver::new(solver::NaiveBranching)),
            Some("DLIS") => Box::new(SatisfactionSolver::new(solver::DLIS)),
            Some("DLCS") => Box::new(SatisfactionSolver::new(solver::DLCS)),
            Some("AMOM") => Box::new(SatisfactionSolver::new(solver::MOM)),
            Some("Jeroslaw-Wang") => Box::new(SatisfactionSolver::new(solver::JeroslawWang)),
            _ => unreachable!(), // already handled by clap
        },

        // FIXME
        Some("cdcl") => {
            let branch: Box<dyn BranchingStrategyFactory> =
                match matches.value_of("cdcl-branching") {
                    Some("VSIDS") => Box::new(solver::cdcl::branching_strategies::VSIDS),
                    _ => unreachable!(), // already handled by clap
                };
            let restart: Box<dyn RestartPolicyFactory> = 
                match matches.value_of("cdcl-restart") {
                    Some("fixed")     => Box::new(solver::cdcl::restart_policies::RestartFixed::default()),
                    Some("geometric") => Box::new(solver::cdcl::restart_policies::RestartGeom::default()),
                    Some("luby")      => Box::new(solver::cdcl::restart_policies::RestartLuby::default()),
                    Some("never")     => Box::new(solver::cdcl::restart_policies::RestartNever),
                    _ => unreachable!(), // already handled by clap
                };
            let learning: Box<dyn LearningSchemeFactory> =
                match matches.value_of("cdcl-learning") {
                    Some("relsat") => Box::new(solver::cdcl::learning_schemes::RelSAT),
                    _ => unreachable!(), // already handled by clap
                };
            let deletion: Box<dyn ClauseDeletionStrategyFactory> = 
                match matches.value_of("cdcl-deletion") {
                    Some("berk-min") => Box::new(solver::cdcl::deletion_strategies::BerkMin::default()),
                    Some("never") => Box::new(solver::cdcl::deletion_strategies::NoDeletion),
                    _ => unreachable!(), // already handled by clap
                };

            let mut preprocessors: Vec<Box<dyn PreprocessorFactory>> = Vec::new();
            match matches.values_of("cdcl-prepoc") {
                Some(procs) => for proc in procs {
                    match proc {
                        "niver" => preprocessors.push(Box::new(NiVER)),
                        "tautologies" => preprocessors.push(Box::new(RemoveTautology)),
                        "empty" => preprocessors.push(Box::new(NoPreprocessing)),
                        _ => unreachable!(),
                    };
                },
                None => ()
            }

            let preproc = ListPreprocessor(preprocessors);

            let drup = matches.value_of("drup").map(|s| PathBuf::from(s));

            Box::new(solver::cdcl::CDCLSolver::new(branch, learning, deletion, restart, preproc, drup))
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
