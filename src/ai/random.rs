
use game;
use game::{GameState, Action};

use rand::{thread_rng, Rng, Rand};
use ai::StatelessAI;

// An AI which executes random legal actions
pub struct RandomSogoAI {}

#[allow(dead_code)]
impl RandomSogoAI {
    pub fn new() -> RandomSogoAI {
        RandomSogoAI {}
    }
}

impl StatelessAI for RandomSogoAI {
    fn action(&self, state : &GameState) -> Action {
        thread_rng().choose(&state.legal_actions)
            .map_or(Action::Surrender, |&a| a.clone())
    }
}
