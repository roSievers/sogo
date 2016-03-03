// This file contains basic definitions for AIs as well as helper function to build them.
// Some example AIs are also contained.
extern crate rand;
use self::rand::{thread_rng, Rng};
use game;

pub enum Move {
    Play {x : i8, y : i8},
    Surrender
}

pub trait SogoAI {
    fn reset_game(&self);
    fn execute_move(&self, state : &game::GameState) -> Move;
}

pub fn run_match<T : SogoAI, U : SogoAI>(structure : &game::GameStructure, white_player : &T, black_player : &U) -> game::GameState {
    let mut i = 0;

    let mut state = game::GameState::new();
    while state.victory_state == game::VictoryState::Undecided {
        if state.age == 64 {
            state.victory_state = game::VictoryState::Draw;
            return state;
        }
        let action = if i % 2 == 0 {white_player.execute_move(&state)} else {black_player.execute_move(&state)};
        match action {
            Move::Play { x, y} => game::play_at(structure, &mut state, x, y),
            Move::Surrender => state.victory_state = game::VictoryState::Win(game::flip_color(state.current_color))
        };
        i += 1;
    }
    // println!("{:?}", i);
    return state;
}

// An AI which executes random legal moves
#[allow(dead_code)] // Empty structs are unstable.
pub struct RandomSogoAI {
    alibi : i8,
}

impl RandomSogoAI {
    pub fn new() -> RandomSogoAI {
        RandomSogoAI { alibi : 42 }
    }
}

impl SogoAI for RandomSogoAI {
    fn reset_game(&self) { }
    fn execute_move(&self, state : &game::GameState) -> Move {
        let position = thread_rng().choose(&state.legal_moves);
        match position {
            Some(&(x, y)) => Move::Play {x:x, y:y},
            None => Move::Surrender
        }
    }
}
