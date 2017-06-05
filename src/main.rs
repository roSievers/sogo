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

use game::{GameStructure};
use ai::StatelessAI;
use constants::LINES; //, PARALLELOGRAMS, PLUSSES};
use std::rc::Rc;

fn main() {
    let matches = parse_command_line_input();

    if let Some(batch_matches) = matches.subcommand_matches("batch") {
        println!("Batch mode isn't implemented yet.");
        return;
    }

    interactive();
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

    App::new("Sogo - Play four in a row.")
        .version("0.0.1")
        .author("Rolf Sievers <rolf.sievers@posteo.de>")
        .about("UI and AIs for Sogo.")
        .subcommand(batch_run)
        .get_matches()
}

fn interactive() {
    let ui_connector = ui::UiConnector::new();

    let structure = Rc::new(GameStructure::new(&LINES));

    // let mut p2 = ai::tree::TreeJudgementAI::new(structure.clone(), 3);
    let mut p2 = ai::mc::MonteCarloAI::new(structure.clone(), 1000);

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

fn user_turn(ui_connector: &ui::UiConnector, state: &mut game::State, structure: &GameStructure) {
    // Wait for the player to make the first action
    let action = ui_connector.wait_for_action().unwrap();

    let color = state.current_color;
    state.execute(&structure, action);
    ui_connector.confirmed_action(action, color).unwrap();
}

fn ai_turn<A: StatelessAI>(ui_connector: &ui::UiConnector,
                      ai: &mut A,
                      state: &mut game::State,
                      structure: &GameStructure) {
    // Let the AI take one action
    let action = ai.action(&state);

    let color = state.current_color;
    state.execute(&structure, action);
    ui_connector.confirmed_action(action, color).unwrap();
}
