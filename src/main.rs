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
mod command_line;

// UI dependencies
extern crate kiss3d;
extern crate glfw;
extern crate nalgebra as na;

extern crate rand;
extern crate clap;

extern crate threadpool;

use ai::StatelessAI;
use std::sync::Arc;

fn main() {
    use command_line::Arguments;

    let argument_result = command_line::get_arguments();

    let argument = match argument_result {
        Ok(value) => value,
        Err(err) => {
            println!("Invalid command line arguments: {}", err);
            return;
        }
    };

    let replay = match argument {
        Arguments::VsAI {
            structure,
            opponent,
        } => interactive(Arc::new(structure.into()), ai::AIBox::new(opponent)),
        Arguments::Batch {
            structure,
            count,
            ai_1,
            ai_2,
        } => {
            batch(
                Arc::new(structure.into()),
                count,
                ai::AIBox::new(ai_1),
                ai::AIBox::new(ai_2),
            );
            // FIXME: Store the replays. Maybe batch can return an Iterator of
            // type History? Or just do the counting in this loop.
            return;
        }
        Arguments::Demo {
            structure,
            ai_1,
            ai_2,
        } => {
            demo(
                Arc::new(structure.into()),
                ai::AIBox::new(ai_1),
                ai::AIBox::new(ai_2),
            )
        }
        Arguments::Humans { structure } => humans(Arc::new(structure.into())),
    };

    // TODO: Store this in a file instead.
    // (Actually use the supplied replay path.)
    println!("{}", replay.notation());
    // let replay_path = matches.value_of("replay-file").unwrap();


}




fn interactive(structure: Arc<game::Structure>, mut p2: ai::AIBox) -> replay::History {
    let ui_connector = ui::UiConnector::new(structure.clone());

    let mut replay = replay::History::new(structure.clone());

    loop {
        user_turn(&ui_connector, &mut replay);

        // Check for victory.
        if !replay.state.victory_state.active() {
            println!("The Human has won the game.");
            ui_connector.game_over(replay.state.victory_state);
            ui_connector.wait_for_halt();
            break;
        }

        ai_turn(&ui_connector, &mut p2, &mut replay);

        // Check for victory.
        if !replay.state.victory_state.active() {
            println!("The AI has won the game.");
            ui_connector.game_over(replay.state.victory_state);
            ui_connector.wait_for_halt();
            break;
        }
    }

    replay
}

fn user_turn(ui_connector: &ui::UiConnector, replay: &mut replay::History) {
    // Wait for the player to make the first action
    let action = ui_connector.wait_for_action().unwrap();

    let color = replay.state.current_color;
    replay.add(action);
    ui_connector.confirmed_action(action, color).unwrap();
}

fn ai_turn<A: StatelessAI>(
    ui_connector: &ui::UiConnector,
    ai: &mut A,
    replay: &mut replay::History,
) {
    // Let the AI take one action
    let action = ai.action(&replay.state);

    let color = replay.state.current_color;
    replay.add(action);
    ui_connector.confirmed_action(action, color).unwrap();
}


// This is simmilar to interactive, but the player isn't allowed to do any moves.
fn demo(
    structure: Arc<game::Structure>,
    mut active_ai: ai::AIBox,
    mut waiting_ai: ai::AIBox,
) -> replay::History {
    use std::mem::swap;
    let ui_connector = ui::UiConnector::new(structure.clone());

    let mut replay = replay::History::new(structure.clone());

    loop {
        ai_turn(&ui_connector, &mut active_ai, &mut replay);

        // Check for victory.
        if !replay.state.victory_state.active() {
            ui_connector.game_over(replay.state.victory_state);
            ui_connector.wait_for_halt();
            break;
        }

        // Swap AIs.
        swap(&mut active_ai, &mut waiting_ai);
    }

    replay
}


/* Batch mode allows you to pitch two AIs against each other
and get some information what happened in the game. */
fn batch(structure: Arc<game::Structure>, count: usize, mut ai_1: ai::AIBox, mut ai_2: ai::AIBox) {
    for i in 1..count + 1 {
        println!(
            "Match {} results in {:?}",
            i,
            ai::run_match(structure.clone(), &mut ai_1, &mut ai_2).victory_state
        );
    }
}

fn humans(structure: Arc<game::Structure>) -> replay::History {
    let ui_connector = ui::UiConnector::new(structure.clone());

    let mut replay = replay::History::new(structure.clone());

    loop {
        user_turn(&ui_connector, &mut replay);

        // Check for victory.
        if !replay.state.victory_state.active() {
            println!("Game Over.");
            ui_connector.game_over(replay.state.victory_state);
            ui_connector.wait_for_halt();
            break;
        }
    }

    replay
}
