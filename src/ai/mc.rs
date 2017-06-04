
use ai::StatelessAI;

use game;
use game::{GameState, Action, GameStructure, PlayerColor, VictoryState, VictoryStats};

use std::rc::Rc;
use rand::{thread_rng, Rng, Rand};


// Pure Monte Carlo AI
// For each possible action, a number of playouts is run.
// This should give an approximate information
// about the value of each action.
#[allow(dead_code)]
pub struct MonteCarloAI {
    endurance: i32, // How many random games am I allowed to play each turn?
    structure: Rc<GameStructure>,
}

#[allow(dead_code)]
impl MonteCarloAI {
    pub fn new(structure: Rc<GameStructure>, endurance: i32) -> MonteCarloAI {
        MonteCarloAI {
            endurance: endurance,
            structure: structure,
        }
    }
}

impl StatelessAI for MonteCarloAI {
    fn action(&self, state: &GameState) -> Action {
        let my_color = state.current_color;
        let endurance_per_action = self.endurance / (state.legal_actions.len() as i32);

        // Each action is judged by running a certain number of random matches.
        // The action with the best win ratio is selected.
        let (&best_action, _) = state
            .legal_actions
            .iter()
            .map(|action| {
                     let new_state = state.execute_action_functional(&self.structure, action);
                     let value = monte_carlo_judgement(&self.structure,
                                                       &new_state,
                                                       my_color,
                                                       endurance_per_action);
                     (action, value)
                 })
            .max_by_key(|&(_, value)| value)
            .unwrap();

        best_action
    }
}

fn monte_carlo_judgement(structure: &GameStructure,
                         state: &GameState,
                         my_color: PlayerColor,
                         amount: i32)
                         -> i32 {
    let stats = random_playout_sample(structure, state, amount);
    if my_color == PlayerColor::White {
        return stats.white - stats.black;
    } else {
        return stats.black - stats.white;
    }
}


pub fn random_playout(structure: &GameStructure, state: &GameState) -> VictoryState {
    let mut my_state = state.clone();
    let mut rng = thread_rng();
    while my_state.victory_state == VictoryState::Undecided {
        let surrender = Action::Surrender;
        let action = rng.choose(&my_state.legal_actions)
            .unwrap_or(&surrender)
            .clone();
        my_state.execute_action(structure, &action);
    }
    return my_state.victory_state;
}


pub fn random_playout_sample(structure: &GameStructure,
                             state: &GameState,
                             number: i32)
                             -> VictoryStats {
    let mut statics = game::VictoryStats::new();
    for _ in 0..number {
        let result = random_playout(&structure, &state);
        match result {
            game::VictoryState::Win(game::PlayerColor::White) => statics.white += 1,
            game::VictoryState::Win(game::PlayerColor::Black) => statics.black += 1,
            game::VictoryState::Draw => statics.draws += 1,
            game::VictoryState::Undecided => {
                panic!("The game_state should never be undecided after a random playout.")
            }
        }
    }
    return statics;
}
