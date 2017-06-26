
use ai;
use ai::value;
use ai::StatelessAI;

use game;
use game::Position2;


pub fn recursive_judgement(
    state: &game::State,
    my_color: game::Color,
    depth: u8,
    value_function: value::Simple,
) -> i32 {
    if depth == 0 || !state.victory_state.active() {
        value_function.value_of(state, my_color)
    } else {
        let values = state.legal_actions().map(|action| {
            let mut new_state = state.clone();
            new_state.execute(action);
            let value = recursive_judgement(&new_state, my_color, depth - 1, value_function);
            value
        });

        if state.current_color == my_color {
            values.max()
        } else {
            values.min()
        }.unwrap()
    }
}

pub struct TreeJudgementAI {
    search_depth: u8,
    value_function: value::Simple,
}

impl TreeJudgementAI {
    pub fn new(depth: u8, value_function: value::Simple) -> TreeJudgementAI {
        TreeJudgementAI {
            search_depth: depth,
            value_function,
        }
    }
}

impl StatelessAI for TreeJudgementAI {
    fn action(&self, state: &game::State) -> Position2 {
        let my_color = state.current_color;

        let graded_actions = state.legal_actions().map(|action| {
            let mut new_state = state.clone();
            new_state.execute(action);
            let value = recursive_judgement(
                &new_state,
                my_color,
                self.search_depth - 1,
                self.value_function,
            );
            (action, value)
        });

        ai::random_best_move(graded_actions)
    }
}
