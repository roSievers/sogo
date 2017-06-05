// This file contains basic definitions for AIs as well as helper function to build them.
// Some example AIs are also contained.

pub mod random;
pub mod mc;
// pub mod tree; //FIXME

use game;
use game::{GameStructure, PlayerColor, VictoryState, VictoryStats, LineState, Action};
use std::rc::Rc;
use helpers::upper_bound_index;

/*pub trait SogoAI {
    fn reset_game(&mut self);
    // Some information may be preserved after an opponent's turn.
    // Tree based algorithms may carry over part of the search tree.
    fn register_opponent_action(&mut self, &Action);
    fn decide_action(&mut self, state : &GameState) -> Action;
        // An imutable reference to the game_state is passed for convenience only.
}*/

// I should first focus on stateless AIs. The current AIs are all stateless
// and I shouldn't have to deal with the extra baggage.
pub trait StatelessAI {
    fn action(&self, state: &game::State) -> Action;
}

/*impl<Ai: StatelessAI> SogoAI for Ai {
    fn reset_game(&mut self) {}
    fn register_opponent_action(&mut self, _ : &Action) {}
    fn decide_action(&mut self, state : &GameState) -> Action {
        self.action(state)
    }
}*/


// FIXME: The tests rely on this.
/*pub fn run_match<T : StatelessAI, U : StatelessAI>(
        structure : &GameStructure, white_player : &mut T, black_player : &mut U)
        -> GameState {
    let mut i = 0;

    let mut state = GameState::new(&structure);
    while state.victory_state == VictoryState::Undecided {
        if state.age == 64 {
            state.victory_state = VictoryState::Draw;
            return state;
        }
        let action = if i % 2 == 0 {white_player.decide_action(&state)} else {black_player.decide_action(&state)};
        if i % 2 == 0 {black_player.register_opponent_action(&action)} else {white_player.register_opponent_action(&action)};
        state.execute_action(structure, &action);
        i += 1;
    }
    // println!("{:?}", i);
    return state;
}*/
