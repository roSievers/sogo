
use ai::value;
use ai::StatelessAI;

use game;
use game::{Action, PlayerColor};
use std::rc::Rc;


pub fn recursive_judgement(structure: &game::Structure,
                           state: &game::State,
                           my_color: PlayerColor,
                           depth: u8,
                           value_function: value::Simple)
                           -> i32 {
    if depth == 0 || !state.victory_state.active() {
        value_function.value_of(structure, state, my_color)
    } else {
        let values = state
            .legal_actions()
            .map(|action| {
                let mut new_state = state.clone();
                new_state.execute(structure, action);
                let value =
                    recursive_judgement(structure, &new_state, my_color, depth - 1, value_function);
                value
            });

        if state.current_color == my_color {
                values.max()
            } else {
                values.min()
            }
            .unwrap()
    }
}

#[allow(dead_code)]
pub struct TreeJudgementAI {
    structure: Rc<game::Structure>,
    search_depth: u8,
    value_function: value::Simple,
}

#[allow(dead_code)]
impl TreeJudgementAI {
    pub fn new(structure: Rc<game::Structure>,
               depth: u8,
               value_function: value::Simple)
               -> TreeJudgementAI {
        TreeJudgementAI {
            structure: structure,
            search_depth: depth,
            value_function,
        }
    }
}

impl StatelessAI for TreeJudgementAI {
    fn action(&self, state: &game::State) -> Action {
        let my_color = state.current_color;

        let (best_action, best_value) = state
            .legal_actions()
            .map(|action| {
                let mut new_state = state.clone();
                new_state.execute(&self.structure, action);
                let value = recursive_judgement(&self.structure,
                                                &new_state,
                                                my_color,
                                                self.search_depth - 1,
                                                self.value_function);
                (action, value)
            })
            .max_by_key(|&(_, value)| value)
            .unwrap();

        // println!("Executing {:?} with value {}.", best_action, best_value);

        best_action
    }
}
