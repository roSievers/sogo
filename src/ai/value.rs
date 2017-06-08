/* This module collects value functions. */

use std::str::FromStr;

use game;
use game::{PlayerColor, LineState};

#[derive(Clone, Copy, Debug)]
pub enum Simple {
    Subsets,
    WinOnly,
}

impl Simple {
    pub fn value_of(self,
                    structure: &game::Structure,
                    state: &game::State,
                    my_color: PlayerColor)
                    -> i32 {
        match self {
            Simple::Subsets => subsets(structure, state, my_color),
            Simple::WinOnly => win_only(structure, state, my_color),
        }
    }
}

impl FromStr for Simple {
    type Err = ();
    fn from_str(s: &str) -> Result<Simple, ()> {
        match s {
            "subsets" => Ok(Simple::Subsets),
            "win" => Ok(Simple::WinOnly),
            _ => Err(())
        }
    }
}

// For each possible winning subset, this adds some score.
// One piece on a line => 1 Point
// Two pieces on a line => 4 Points
// Three pieces on a line => 9 Points
pub fn subsets(structure: &game::Structure, state: &game::State, my_color: PlayerColor) -> i32 {
    if let game::VictoryState::Win { winner, .. } = state.victory_state {
        if winner == my_color {
            return 1000;
        } else {
            return -1000;
        }
    }

    let mut score = 0;

    for subset in &structure.source {
        score += match subset.win_state(state) {
            LineState::Empty => 0,
            LineState::Mixed => 0,
            LineState::Pure { color, count } => {
                if color == my_color {
                    (count * count) as i32
                } else {
                    -(count * count) as i32
                }
            }
            LineState::Win(_) => {
                panic!("If the game is already won this should be caught earlier.")
            }
        }
    }

    score
}

#[test]
fn test_subsets_values() {
    use game::{Structure, State, Position2};
    use game::PlayerColor::White;
    use constants::LINES;

    let structure = Structure::new(&LINES);

    let mut state = State::new();
    assert_eq!(0, subsets(&structure, &state, White));

    state.insert(&structure, Position2::new(0, 0));
    assert_eq!(7, subsets(&structure, &state, White));

    state.insert(&structure, Position2::new(0, 3));
    assert_eq!(0, subsets(&structure, &state, White));
}

// This value function only checks if you won the game already.
pub fn win_only(structure: &game::Structure, state: &game::State, my_color: PlayerColor) -> i32 {
    if let game::VictoryState::Win { winner, .. } = state.victory_state {
        if winner == my_color {
            return 1;
        } else {
            return -1;
        }
    } else {
        0
    }
}
