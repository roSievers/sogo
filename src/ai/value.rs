/* This module collects value functions. */

use std::str::FromStr;

use game;
use game::{LineState, Position3, Position2};

#[derive(Clone, Copy, Debug)]
pub enum Simple {
    Subsets,
    WinOnly,
}

impl Simple {
    pub fn value_of(self, state: &game::State, my_color: game::Color) -> i32 {
        match self {
            Simple::Subsets => subsets(state, my_color),
            Simple::WinOnly => win_only(state, my_color),
        }
    }
}

impl FromStr for Simple {
    type Err = ();
    fn from_str(s: &str) -> Result<Simple, ()> {
        match s {
            "subsets" => Ok(Simple::Subsets),
            "win" => Ok(Simple::WinOnly),
            _ => Err(()),
        }
    }
}

// For each possible winning subset, this adds some score.
// One piece on a line => 1 Point
// Two pieces on a line => 4 Points
// Three pieces on a line => 9 Points
pub fn subsets(state: &game::State, my_color: game::Color) -> i32 {
    if let game::VictoryState::Win { winner, .. } = state.victory_state {
        if winner == my_color {
            return 1000;
        } else {
            return -1000;
        }
    }

    let mut score = 0;

    for subset in &state.structure.source {
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
    use game::Color::White;
    use std::sync::Arc;
    use constants::LINES;

    let structure = Structure::new(&LINES);

    let mut state = State::new(Arc::new(structure));
    assert_eq!(0, subsets(&state, White));

    state.insert(Position2::new(0, 0));
    assert_eq!(7, subsets(&state, White));

    state.insert(Position2::new(0, 3));
    assert_eq!(0, subsets(&state, White));
}

// This value function only checks if you won the game already.
pub fn win_only(state: &game::State, my_color: game::Color) -> i32 {
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


/* New idea/project for a value function. Give each point a value, then each
column and in conclusion, give one to the whole board (if desired). This can also
be used to pick which actions to inspect further. */

// How much a particular action is worth to a player. The LastMissingPiece
// variant indicates that playing this action wins the game.
// The DirectLoss variant indicates that the player is sure to loose by playing this.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum SideValue {
    // Do I win if I place here?
    LastMissingPiece,
    // Or do I at least get a piece which aligns with a lot of other pieces?
    Heuristic(i32),
    DirectLoss,
}

// Calculates the point value for White and Black.
#[allow(dead_code)]
fn point_value(state: &game::State, position: Position3) -> Option<(SideValue, SideValue)> {
    use game::Color::White;

    // This is only defined for empty positions.
    if let game::PointState::Piece(_) = state.at(position) {
        return None;
    }

    let mut white_value = 0;
    let mut black_value = 0;
    let mut last_white_piece = false;
    let mut last_black_piece = false;

    for subset_index in &state.structure.reverse[position.0 as usize] {
        let subset = state.structure.source[*subset_index];
        let win_state = subset.win_state(state);
        match win_state {
            game::LineState::Empty => {
                white_value += 1;
                black_value += 1;
            }
            game::LineState::Pure { color, count } => {
                let total_count = count as i32 + 1;
                if total_count == state.structure.object_size as i32 {
                    if color == White {
                        last_white_piece = true;
                    } else {
                        last_black_piece = true;
                    }
                } else {
                    if color == White {
                        white_value += total_count * total_count;
                    } else {
                        black_value += total_count * total_count;
                    }
                }
            }
            game::LineState::Mixed => {}
            game::LineState::Win(_) => {
                // We made sure that there is at least one empty position.
                unreachable!();
            }
        }
    }

    let white_point_value = if last_white_piece {
        SideValue::LastMissingPiece
    } else {
        SideValue::Heuristic(white_value)
    };

    let black_point_value = if last_black_piece {
        SideValue::LastMissingPiece
    } else {
        SideValue::Heuristic(black_value)
    };

    Some((white_point_value, black_point_value))
}

// Is this column worth playing at?
#[allow(dead_code)]
fn column_value(state: &game::State, position: Position2) -> Option<(SideValue, SideValue)> {
    use self::SideValue::{LastMissingPiece, Heuristic, DirectLoss};
    let height: u8 = state.column_height[position.0 as usize];

    match height {
        4 => None,
        3 => point_value(state, position.with_height(3)),
        h => {
            let (white_placement_value, black_placement_value) =
                point_value(state, position.with_height(h)).unwrap();
            let (white_response_value, black_response_value) =
                point_value(state, position.with_height(h + 1)).unwrap();

            let white_value = {
                if let Heuristic(w) = white_placement_value {
                    if let Heuristic(b) = black_response_value {
                        Heuristic(w - b)
                    } else {
                        DirectLoss
                    }
                } else {
                    LastMissingPiece
                }
            };

            let black_value = {
                if let Heuristic(b) = black_placement_value {
                    if let Heuristic(w) = white_response_value {
                        Heuristic(b - w)
                    } else {
                        DirectLoss
                    }
                } else {
                    LastMissingPiece
                }
            };

            Some((white_value, black_value))
        }
    }
}
