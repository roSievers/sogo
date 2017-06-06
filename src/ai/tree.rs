
use ai::StatelessAI;

use game;
use game::{Action, GameStructure, PlayerColor, LineState};
use std::rc::Rc;


pub fn easy_judgement(structure: &GameStructure,
                      state: &game::State,
                      my_color: PlayerColor)
                      -> i32 {
    let mut score = 0;

    for subset in &structure.source {
        score += match subset.win_state(state) {
            LineState::Empty => 0,
            LineState::Win(color) => 1000 * (if color == my_color { 1 } else { -1 }),
            // If I'm still allowed to play, that must have been my win.
            LineState::Mixed => 0,
            LineState::Pure { color, count } => {
                (count * count * (if color == my_color { 1 } else { -1 })) as i32
            }
        }
    }

    score
}

pub fn recursive_judgement(structure: &GameStructure,
                           state: &game::State,
                           my_color: PlayerColor,
                           depth: i8)
                           -> i32 {
    if depth == 0 {
        easy_judgement(structure, state, my_color)
    } else {
        let values = state
            .legal_actions()
            .map(|action| {
                     let mut new_state = state.clone();
                     new_state.execute(structure, action);
                     let value = recursive_judgement(structure, &new_state, my_color, depth - 1);
                     value
                 });

        if state.current_color == my_color {
            values.max()
        } else {
            values.min()
        }.unwrap_or_else(|| easy_judgement(structure, state, my_color))
    }
}

#[allow(dead_code)]
pub struct TreeJudgementAI {
    structure: Rc<game::GameStructure>,
    search_depth: i8,
}

#[allow(dead_code)]
impl TreeJudgementAI {
    pub fn new(structure: Rc<GameStructure>, depth: i8) -> TreeJudgementAI {
        TreeJudgementAI {
            structure: structure,
            search_depth: depth,
        }
    }
}

impl StatelessAI for TreeJudgementAI {
    fn action(&self, state: &game::State) -> Action {
        let my_color = state.current_color;

        let (best_action, best_value) = state.legal_actions()
            .map(|action| {
                     let mut new_state = state.clone();
                     new_state.execute(&self.structure, action);
                     let value = recursive_judgement(&self.structure,
                                                       &new_state,
                                                       my_color,
                                                       self.search_depth - 1);
                     (action, value)
                 })
            .max_by_key(|&(_, value)| value)
            .unwrap();

        println!("Executing {:?} with value {}.", best_action, best_value);

        best_action
    }
}
