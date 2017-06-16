
use game;
use game::Action;

use rand::{thread_rng, Rng};
use ai::StatelessAI;

// An AI which executes random legal actions
#[allow(dead_code)]
pub struct RandomSogoAI {}

#[allow(dead_code)]
impl RandomSogoAI {
    pub fn new() -> RandomSogoAI {
        RandomSogoAI {}
    }
}

impl StatelessAI for RandomSogoAI {
    fn action(&self, state: &game::State) -> Action {
        let legal_actions: Vec<Action> = state.legal_actions().collect();
        thread_rng().choose(&legal_actions).map_or(
            Action::Surrender,
            |&a| a.clone(),
        )
    }
}
