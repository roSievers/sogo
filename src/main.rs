#[cfg(test)]
mod tests;

mod game;
mod ai;
mod ui;
mod game_view;
mod constants;
mod helpers;
mod thread_synchronisation;
mod replay;

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

    fn batch(arguments: &clap::ArgMatches) -> Result<(), String> {
        let ai1_params = arguments.values_of("ai1").map(ai_parser).unwrap()?;
        let ai2_params = arguments.values_of("ai2").map(ai_parser).unwrap()?;
        let count: usize = arguments
            .value_of("count")
            .unwrap()
            .parse::<usize>()
            .unwrap();

        let structure = Rc::new(game::Structure::new(&LINES));

        let mut ai1 = ai::AIBox::new(structure.clone(), ai1_params);
        let mut ai2 = ai::AIBox::new(structure.clone(), ai2_params);

        for i in 1..count + 1 {
            println!(
                "Match {} results in {:?}",
                i,
                ai::run_match(structure.clone(), &mut ai1, &mut ai2).victory_state
            );
        }
        Ok(())
    }

    if let Some(batch_matches) = matches.subcommand_matches("batch") {
        batch(batch_matches).unwrap();
        return;
    }

    if let Some(demo_matches) = matches.subcommand_matches("demo") {
        demo(demo_matches).unwrap();
        return;
    }

    let ai_parameter_result = matches.values_of("opponent").map(ai_parser).unwrap_or(Ok(
        ai::Constructor::MonteCarlo { endurance: 1000 },
    ));

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
                .validator(validate_integer),
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
        .subcommand(batch_run)
        .subcommand(demo_match)
        .arg(opponent)
        .get_matches()
}


fn ai_parser(mut values: clap::Values) -> Result<ai::Constructor, String> {
    let ai_name: &str = values.next().unwrap();
    match ai_name {
        "random" => Ok(ai::Constructor::Random),
        "mc" => {
            let endurance = values.next().unwrap_or("10000").parse::<usize>().map_err(
                |_| "The endurance needs to be a number.",
            )?;
            Ok(ai::Constructor::MonteCarlo { endurance })
        }
        "tree" => {
            let depth = values.next().unwrap_or("2").parse::<u8>().map_err(
                |_| "The depth needs to be a number.",
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
        _ => Err("AI not recognized.")?,
    }
}

fn interactive(structure: Rc<game::Structure>, mut p2: ai::AIBox) {
    let ui_connector = ui::UiConnector::new();

    // Run a game, this should look synchronous.
    let mut state = game::State::new(structure.clone());

    loop {
        user_turn(&ui_connector, &mut state);

        // Check for victory.
        if !state.victory_state.active() {
            println!("The Human has won the game.");
            ui_connector.game_over(state.victory_state);
            ui_connector.wait_for_halt();
            break;
        }

        ai_turn(&ui_connector, &mut p2, &mut state);

        // Check for victory.
        if !state.victory_state.active() {
            println!("The AI has won the game.");
            ui_connector.game_over(state.victory_state);
            ui_connector.wait_for_halt();
            break;
        }
    }
}

fn user_turn(ui_connector: &ui::UiConnector, state: &mut game::State) {
    // Wait for the player to make the first action
    let action = ui_connector.wait_for_action().unwrap();

    let color = state.current_color;
    state.execute(action);
    ui_connector.confirmed_action(action, color).unwrap();
}

fn ai_turn<A: StatelessAI>(ui_connector: &ui::UiConnector, ai: &mut A, state: &mut game::State) {
    // Let the AI take one action
    let action = ai.action(&state);

    let color = state.current_color;
    state.execute(action);
    ui_connector.confirmed_action(action, color).unwrap();
}


// This is simmilar to interactive, but the player isn't allowed to do any moves.
fn demo(arguments: &clap::ArgMatches) -> Result<(), String> {
    let ai1_params = arguments.values_of("ai1").map(ai_parser).unwrap()?;
    let ai2_params = arguments.values_of("ai2").map(ai_parser).unwrap()?;

    let structure = Rc::new(game::Structure::new(&LINES));

    let mut active_ai = ai::AIBox::new(structure.clone(), ai1_params);
    let mut waiting_ai = ai::AIBox::new(structure.clone(), ai2_params);

    let ui_connector = ui::UiConnector::new();

    let mut state = game::State::new(structure.clone());

    loop {
        ai_turn(&ui_connector, &mut active_ai, &mut state);

        // Check for victory.
        if !state.victory_state.active() {
            ui_connector.game_over(state.victory_state);
            ui_connector.wait_for_halt();
            break;
        }

        // Swap AIs.
        let temp = active_ai;
        active_ai = waiting_ai;
        waiting_ai = temp;
    }

    Ok(())
}
