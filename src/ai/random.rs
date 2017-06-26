
use game;
use game::Position2;

use rand::{thread_rng, Rng};
use ai::StatelessAI;

// An AI which executes random legal actions
pub struct RandomSogoAI {}

impl RandomSogoAI {
    pub fn new() -> RandomSogoAI {
        RandomSogoAI {}
    }
}

impl StatelessAI for RandomSogoAI {
    fn action(&self, state: &game::State) -> Position2 {
        let legal_actions: Vec<Position2> = state.legal_actions().collect();
        *thread_rng().choose(&legal_actions).unwrap()
    }
}
