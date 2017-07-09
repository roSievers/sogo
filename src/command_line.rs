// Command line argument parser

use clap;
use clap::{App, Arg, SubCommand};

use std::str::FromStr;

use ai;
use constants;

pub enum Arguments {
    VsAI {
        structure: constants::StructureSource,
        opponent: ai::Constructor,
    },
    Demo {
        structure: constants::StructureSource,
        ai_1: ai::Constructor,
        ai_2: ai::Constructor,
    },
    Batch {
        structure: constants::StructureSource,
        ai_1: ai::Constructor,
        ai_2: ai::Constructor,
        count: usize,
    },
    Humans { structure: constants::StructureSource, },
}

pub fn get_arguments() -> Result<Arguments, String> {
    let matches = setup_clap();

    let structure = matches
        .value_of("structure")
        .unwrap()
        .parse::<constants::StructureSource>()
        .unwrap();

    if let Some(batch_matches) = matches.subcommand_matches("batch") {
        let ai_1 = batch_matches.values_of("ai1").map(ai_parser).unwrap()?;
        let ai_2 = batch_matches.values_of("ai2").map(ai_parser).unwrap()?;
        let count: usize = batch_matches
            .value_of("count")
            .unwrap()
            .parse::<usize>()
            .unwrap();

        Ok(Arguments::Batch {
            structure,
            ai_1,
            ai_2,
            count,
        })
    } else if let Some(demo_matches) = matches.subcommand_matches("demo") {
        let ai_1 = demo_matches.values_of("ai1").map(ai_parser).unwrap()?;
        let ai_2 = demo_matches.values_of("ai2").map(ai_parser).unwrap()?;

        Ok(Arguments::Demo {
            structure,
            ai_1,
            ai_2,
        })
    } else if let Some(_) = matches.subcommand_matches("human") {
        Ok(Arguments::Humans { structure })
    } else {
        // No subcommand is activated, this is a normal game VS the AI.
        let opponent = match matches.values_of("opponent") {
            Some(description) => ai_parser(description),
            None => Ok(ai::Constructor::MonteCarlo { endurance: 1000 }),
        }?;

        Ok(Arguments::VsAI {
            structure,
            opponent,
        })
    }
}

fn validate_parse<T: FromStr>(s: String) -> Result<(), String> {
    match s.parse::<T>() {
        Ok(_) => Ok(()),
        Err(_) => Err("Cannot parse this argument.".to_owned()),
    }
}

fn setup_clap<'clap>() -> clap::ArgMatches<'clap> {
    let ai_1 = || {
        Arg::with_name("ai1")
            .short("p")
            .required(true)
            .help("Specify first AI.")
            .min_values(1)
    };
    let ai_2 = || {
        Arg::with_name("ai2")
            .short("q")
            .required(true)
            .help("Specify second AI.")
            .min_values(1)
    };

    let batch_run = SubCommand::with_name("batch")
        .about("Executes many AI matches at once.")
        .arg(
            Arg::with_name("count")
                .short("n")
                .long("count")
                .help("How many matches should be played")
                .takes_value(true)
                .default_value("1")
                .validator(validate_parse::<u32>),
        )
        .arg(ai_1())
        .arg(ai_2());

    let demo_match = SubCommand::with_name("demo")
        .about("Demonstration match with two AIs")
        .arg(ai_1())
        .arg(ai_2());

    let opponent = Arg::with_name("opponent")
        .short("p")
        .long("player")
        .help("Specify which AI you want to play against.")
        .min_values(1);

    App::new("Sogo - Play four in a row.")
        .version("0.0.1")
        .author("Rolf Sievers <rolf.sievers@posteo.de>")
        .about("UI and AIs for Sogo.")
        .arg(
            Arg::with_name("structure")
                .short("s")
                .long("structure")
                .help("Specify which rules you want to play by.")
                .default_value("line")
                .validator(validate_parse::<constants::StructureSource>),
        )
        .subcommand(batch_run)
        .subcommand(demo_match)
        .subcommand(SubCommand::with_name("human").about(
            "Allow two humans to play against each other.",
        ))
        .arg(opponent)
        .arg(
            Arg::with_name("replay-file")
                .short("r")
                .long("replay")
                .help("Where the replay file should be stored.")
                .default_value("replay.sogo"),
        )
        .get_matches()
}


fn ai_parser(mut values: clap::Values) -> Result<ai::Constructor, String> {
    let ai_name: &str = values.next().unwrap();
    match ai_name {
        "random" => Ok(ai::Constructor::Random),
        "mc" => {
            let endurance = values.next().unwrap_or("10000").parse::<usize>().map_err(
                |_| "The endurance needs to be a positive integer.",
            )?;
            Ok(ai::Constructor::MonteCarlo { endurance })
        }
        "tree" => {
            let depth = values.next().unwrap_or("2").parse::<u8>().map_err(
                |_| "The depth needs to be a small positive integer.",
            )?;

            let value_function = values
                .next()
                .unwrap_or("subsets")
                .parse::<ai::value::Simple>()
                .map_err(|_| "Invalid value function provided.")?;
            Ok(ai::Constructor::Tree {
                depth,
                value_function,
            })
        }
        "mctree" => {
            let endurance = values.next().unwrap_or("10000").parse::<usize>().map_err(
                |_| "The endurance needs to be a positive integer.",
            )?;

            let exploration = values.next().unwrap_or("1.41").parse::<f32>().map_err(
                |_| "The exploration needs to be a positive real number.",
            )?;

            Ok(ai::Constructor::MonteCarloTree {
                endurance,
                exploration,
            })

        }
        _ => Err("AI not recognized.")?,
    }
}
