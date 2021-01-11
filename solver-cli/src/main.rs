mod config;

use clap::{App, Arg};
use config::Config;

fn make_config<'a>() -> Config<'a> {
    let matches = App::new("satisfaction")
        .version("1.0")
        .author("Alex&Korbi&Radek inc.")
        .about("A tool to satisfy all your desires (or prove they are impossible)")
        .arg(Arg::with_name("input")
             .short("l")
             .long("input")
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

    Config{
        input: matches.value_of("input").map(String::from),
        return_code: matches.is_present("return_code"),
        plot: matches.is_present("plot"),
        algorithm: match matches.value_of("algorithm").unwrap() {
            "bruteforce" => panic!("Not supported"),
            "cadical" => panic!("Not supported"),
            "satisfaction" => panic!("Not supported"),
            _ => panic!("Unknown algorithm")
        }
    }
}


fn main() {
    let _config = make_config();
}
