#[cfg(test)]
mod tests;

mod game;
mod ai;
//mod ai_box;
mod human_ai;
mod ui;
mod constants;
mod helpers;
mod thread_synchronisation;

extern crate kiss3d;
extern crate glfw;
extern crate nalgebra as na;

// mod general_analysis;
use game::{GameStructure, GameState, PlayerColor};
use thread_synchronisation::{UiEvent, CoreEvent};
use ai::{SogoAI, TreeJudgementAI};
use constants::{LINES}; //, PARALLELOGRAMS, PLUSSES};
use std::rc::Rc;
use std::thread;
use std::sync::mpsc::{channel};

fn main() {

    let (core_sender, core_receiver) = channel();
    let (ui_sender, ui_receiver) = channel();

    ui_sender.send(UiEvent::EmptyEvent).unwrap();
    let core_sender_ui = core_sender.clone();

    thread::spawn(move|| {
        ui::run_ui(core_sender_ui, ui_receiver);
    });

    let structure = Rc::new(GameStructure::new(&LINES));
    let mut state = GameState::new(&structure);

    // let mut p2 = TreeJudgementAI::new(structure.clone(), 3);
    let mut p2 = ai::MonteCarloAI::new(structure.clone(), 10000);

    // Block the thread until an event arrives, then unwraps and process it.
    // If all senders are destroyed, the while loop will quit, as it receives a RecvError.
    'event_loop: while let Ok(event) = core_receiver.recv() {

        match event {
            CoreEvent::DebugOutput(text) => println!("{}", text),
            CoreEvent::Action {action, color} => {
                if state.current_color == color {
                    state.execute_action(&structure, &action);
                    ui_sender.send(UiEvent::RenderAction{action : action, color : color}).unwrap();
                    if !state.victory_state.active() {
                        ui_sender.send(UiEvent::GameOver(state.victory_state)).unwrap();
                    }
                    if color == PlayerColor::White {
                        // This is a hack to detect if it was a player move.

                        // Ask the enemy AI to move
                        let enemy_action = p2.decide_action(&state);
                        core_sender.send(CoreEvent::Action{action : enemy_action, color : state.current_color}).unwrap();
                    }
                } else {
                    panic!("A color was played out of turn!");
                }
            },
            CoreEvent::Halt => {
                break 'event_loop;
            }
        }
    }
}
