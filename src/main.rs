#[cfg(test)]
mod tests;

mod game;
mod ai;
mod tree_expand_ai;
mod ui;
mod game_view;
mod constants;
mod helpers;
mod thread_synchronisation;

// Command line argument parser
extern crate clap;
use clap::{App, Arg, SubCommand};

// UI dependencies
extern crate kiss3d;
extern crate glfw;
extern crate nalgebra as na;

extern crate rand;

use ai::StatelessAI;
use constants::LINES; //, PARALLELOGRAMS, PLUSSES};
use std::rc::Rc;

fn main() {
    let matches = parse_command_line_input();

    if let Some(batch_matches) = matches.subcommand_matches("batch") {
        println!("Batch mode isn't implemented yet.");
        println!("Arguments: {:?}", batch_matches);
        return;
    }

    let ai_parameter_result = matches
        .values_of("player2ai")
        .map(ai_parser)
        .unwrap_or(Ok(ai::Constructor::MonteCarlo { endurance: 1000 }));

    let ai_parameter = match ai_parameter_result {
        Ok(param) => param,
        Err(error) => {
            println!("Faulty AI specified: {}", error);
            return;
        }
    };

    let structure = Rc::new(game::Structure::new(&LINES));
    let player_2_ai = ai::AIBox::new(structure.clone(), ai_parameter);

    interactive(structure, player_2_ai);
}

fn parse_command_line_input<'clap>() -> clap::ArgMatches<'clap> {
    let validate_integer = |s: String| match s.parse::<u32>() {
        Ok(_) => Ok(()),
        Err(_) => Err("Needs to be an integer.".to_owned()),
    };

    let batch_run = SubCommand::with_name("batch")
        .about("Executes many AI matches at once.")
        .arg(Arg::with_name("count")
                 .short("n")
                 .long("count")
                 .help("How many matches should be played")
                 .takes_value(true)
                 .default_value("1")
                 .validator(validate_integer));

    let p2_ai = Arg::with_name("player2ai")
        .short("p")
        .long("player")
        .help("Specify which AI you want to play against.")
        .min_values(1);

    App::new("Sogo - Play four in a row.")
        .version("0.0.1")
        .author("Rolf Sievers <rolf.sievers@posteo.de>")
        .about("UI and AIs for Sogo.")
        .subcommand(batch_run)
        .arg(p2_ai)
        .get_matches()
}


fn ai_parser(mut values: clap::Values) -> Result<ai::Constructor, String> {
    let ai_name: &str = values.next().unwrap();
    match ai_name {
        "random" => Ok(ai::Constructor::Random),
        "mc" => {
            let endurance = values
                .next()
                .unwrap_or("10000")
                .parse::<usize>()
                .map_err(|_| "The endurance needs to be a number.")?;
            Ok(ai::Constructor::MonteCarlo { endurance })
        }
        "tree" => {
            let depth = values
                .next()
                .unwrap_or("2")
                .parse::<u8>()
                .map_err(|_| "The depth needs to be a number.")?;
            Ok(ai::Constructor::Tree { depth })
        }
        _ => Err("AI not recognized.")?,
    }
}

fn interactive(structure: Rc<game::Structure>, mut p2: ai::AIBox) {
    let ui_connector = ui::UiConnector::new();

    // let mut p2 = ai::tree::TreeJudgementAI::new(structure.clone(), 2);
    // let mut p2 = ai::mc::MonteCarloAI::new(structure.clone(), 10000);
    // let mut p2 = ai::random::RandomSogoAI::new();

    // Run a game, this should look synchronous.
    let mut state = game::State::new();

    loop {
        user_turn(&ui_connector, &mut state, &structure);

        // Check for victory.
        if !state.victory_state.active() {
            println!("The Human has won the game.");
            break;
        }

        ai_turn(&ui_connector, &mut p2, &mut state, &structure);

        // Check for victory.
        if !state.victory_state.active() {
            println!("The AI has won the game.");
            break;
        }
    }
}

fn user_turn(ui_connector: &ui::UiConnector,
             state: &mut game::State,
             structure: &game::Structure) {
    // Wait for the player to make the first action
    let action = ui_connector.wait_for_action().unwrap();

    let color = state.current_color;
    state.execute(&structure, action);
    ui_connector.confirmed_action(action, color).unwrap();
}

fn ai_turn<A: StatelessAI>(ui_connector: &ui::UiConnector,
                           ai: &mut A,
                           state: &mut game::State,
                           structure: &game::Structure) {
    // Let the AI take one action
    let action = ai.action(&state);

    let color = state.current_color;
    state.execute(&structure, action);
    ui_connector.confirmed_action(action, color).unwrap();
}
