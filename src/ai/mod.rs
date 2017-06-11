// This file contains basic definitions for AIs as well as helper function to build them.
// Some example AIs are also contained.

pub mod random;
pub mod mc;
pub mod tree;
pub mod value;

use game;
use game::{Action};
use std::rc::Rc;

// I should first focus on stateless AIs. The current AIs are all stateless
// and I shouldn't have to deal with the extra baggage.
pub trait StatelessAI {
    fn action(&self, state: &game::State) -> Action;
}

#[derive(Clone, Copy, Debug)]
pub enum Constructor {
    Random,
    MonteCarlo { endurance: usize },
    Tree { depth: u8, value_function: value::Simple },
}

pub enum AIBox {
    Random(random::RandomSogoAI),
    MC(mc::MonteCarloAI),
    Tree(tree::TreeJudgementAI),
}

impl AIBox {
    pub fn new(structure: Rc<game::Structure>, ai_parameter: Constructor) -> AIBox {
        match ai_parameter {
            Constructor::Random => AIBox::Random(random::RandomSogoAI::new()),
            Constructor::MonteCarlo { endurance } => {
                AIBox::MC(mc::MonteCarloAI::new(structure.clone(), endurance))
            }
            Constructor::Tree { depth, value_function } => {
                AIBox::Tree(tree::TreeJudgementAI::new(structure.clone(), depth, value_function))
            }
        }
    }
}

impl StatelessAI for AIBox {
    fn action(&self, state: &game::State) -> Action {
        match self {
            &AIBox::Random(ref ai) => ai.action(state),
            &AIBox::MC(ref ai) => ai.action(state),
            &AIBox::Tree(ref ai) => ai.action(state),
        }
    }
}

/*pub trait SogoAI {
    fn reset_game(&mut self);
    // Some information may be preserved after an opponent's turn.
    // Tree based algorithms may carry over part of the search tree.
    fn register_opponent_action(&mut self, &Action);
    fn decide_action(&mut self, state : &GameState) -> Action;
        // An imutable reference to the game_state is passed for convenience only.
}*/


/*impl<Ai: StatelessAI> SogoAI for Ai {
    fn reset_game(&mut self) {}
    fn register_opponent_action(&mut self, _ : &Action) {}
    fn decide_action(&mut self, state : &GameState) -> Action {
        self.action(state)
    }
}*/


pub fn run_match<T : StatelessAI, U : StatelessAI>(
        structure : Rc<game::Structure>, white_player : &mut T, black_player : &mut U)
        -> game::State {
    let mut i = 0;

    let mut state = game::State::new(structure.clone());
    while state.victory_state == game::VictoryState::Undecided {
        if state.age == 64 {
            state.victory_state = game::VictoryState::Draw;
            return state;
        }
        let action = if i % 2 == 0 {white_player.action(&state)} else {black_player.action(&state)};
        state.execute(action);
        i += 1;
    }
    // println!("{:?}", i);
    return state;
}


// To make the gameplay more interesting, the AI should chose a random best move
// instead of a deterministic one.
fn random_best_move<T: Iterator<Item=(Action, i32)>>(mut tuples: T) -> Action {
    use rand::{thread_rng, Rng};

    let (initial_action, initial_value) = tuples.next().unwrap();

    let mut best_actions = vec!(initial_action);
    let mut best_value = initial_value;

    while let Some((action, value)) = tuples.next() {
        if value < best_value {
            continue;
        } else if value == best_value {
            best_actions.push(action);
        } else {
            best_actions = vec!(action);
            best_value = value;
        }
    }

    *thread_rng().choose(&best_actions).unwrap()
}
