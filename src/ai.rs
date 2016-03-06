// This file contains basic definitions for AIs as well as helper function to build them.
// Some example AIs are also contained.
extern crate rand;
use self::rand::{thread_rng, Rng};
use game;
use game::{GameState, GameStructure, VictoryState, LineState};

#[derive(Debug)]
pub enum Move {
    Play {x : i8, y : i8},
    Surrender
}

pub trait SogoAI {
    fn reset_game(&self);
    fn execute_move(&self, state : &GameState) -> Move;
}

pub fn run_match<T : SogoAI, U : SogoAI>(structure : &GameStructure, white_player : &T, black_player : &U) -> GameState {
    let mut i = 0;

    let mut state = GameState::new();
    while state.victory_state == VictoryState::Undecided {
        if state.age == 64 {
            state.victory_state = VictoryState::Draw;
            return state;
        }
        let action = if i % 2 == 0 {white_player.execute_move(&state)} else {black_player.execute_move(&state)};
        match action {
            Move::Play { x, y} => game::play_at(structure, &mut state, x, y),
            Move::Surrender => state.victory_state = VictoryState::Win(!state.current_color)
        };
        i += 1;
    }
    // println!("{:?}", i);
    return state;
}

// An AI which executes random legal moves
#[allow(dead_code)] // Empty structs are unstable.
pub struct RandomSogoAI {
    alibi : i8,
}

impl RandomSogoAI {
    pub fn new() -> RandomSogoAI {
        RandomSogoAI { alibi : 42 }
    }
}

impl SogoAI for RandomSogoAI {
    fn reset_game(&self) { }
    fn execute_move(&self, state : &GameState) -> Move {
        let position = thread_rng().choose(&state.legal_moves);
        // Rust also implements a faster random generator, but it needs to be stored outside of this
        // small function. Caching the RNG might help anyways.
        match position {
            Some(&(x, y)) => Move::Play {x:x, y:y},
            None => Move::Surrender
        }
    }
}

pub struct EasyJudgementAI {
    structure : game::GameStructure,
}

impl EasyJudgementAI {
    pub fn new() -> EasyJudgementAI {
        EasyJudgementAI { structure : game::GameStructure::new() }
    }
}

impl SogoAI for EasyJudgementAI {
    fn reset_game(&self) {}
    fn execute_move(&self, state : &GameState) -> Move {
        // Go through all available moves and jude the outcome.
        let mut best_move = Move::Surrender;
        let mut best_score : i32 = -1000; // If no move has a better score, just surrender.
        let my_color = state.current_color;
        for play in &state.legal_moves {
            let (x, y) = play.clone();
            let mut my_state = state.clone();
            game::play_at(&self.structure, &mut my_state, x, y);
            let mut score : i32 = 0;
            for i in 0..76 {
                let line = my_state.lines[i];
                score += match line {
                    LineState::Empty  => 0,
                    LineState::Win(_) => 1000, // If I'm still allowed to play, that must have been my win.
                    LineState::Mixed  => 0,
                    LineState::Pure { color, count } =>
                        (count * count * (if color == my_color {1} else {-1})) as i32,
                }
            }
            if score > best_score {
                best_move = Move::Play {x:x, y:y};
                best_score = score;
            }
            //println!("{:?} - {:?} -> {:?}", score, best_score, best_move);
        }
        best_move
    }
}

pub struct NestedJudgementAI {
    opponent_ai : EasyJudgementAI,
}

impl NestedJudgementAI {
    pub fn new() -> NestedJudgementAI {
        NestedJudgementAI { opponent_ai : EasyJudgementAI::new()}
    }
}

impl SogoAI for NestedJudgementAI {
    fn reset_game(&self) {}
    fn execute_move(&self, state : &GameState) -> Move {
        // Go through all available moves and jude the outcome.
        let mut best_move = Move::Surrender;
        let mut best_score : i32 = -1000; // If no move has a better score, just surrender.
        let my_color = state.current_color;
        for play in &state.legal_moves {
            let (x, y) = play.clone();
            let mut my_state = state.clone();
            game::play_at(&self.opponent_ai.structure, &mut my_state, x, y);
            // Here is the important difference, we allow the opponent_ai to make a move.
            let action = self.opponent_ai.execute_move(&my_state);
            match action {
                Move::Play { x:x2, y:y2} => game::play_at(&self.opponent_ai.structure, &mut my_state, x2, y2),
                Move::Surrender => my_state.victory_state = VictoryState::Win(!state.current_color)
            };

            let mut score : i32 = 0;
            for i in 0..76 {
                let line = my_state.lines[i];
                score += match line {
                    LineState::Empty  => 0,
                    LineState::Win(color) => 1000 * (if color == my_color {1} else {-1}), // If I'm still allowed to play, that must have been my win.
                    LineState::Mixed  => 0,
                    LineState::Pure { color, count } =>
                        (count * count * (if color == my_color {1} else {-1})) as i32,
                }
            }
            if score > best_score {
                best_move = Move::Play {x:x, y:y};
                best_score = score;
            }
            //println!("{:?} - {:?} -> {:?}", score, best_score, best_move);
        }
        best_move
    }
}

// Monte Carlo Tree search

struct MCNode {
    state : GameState,
    children : MCBranching,
    play_count : i32,
    victory_count : i32,
}

enum MCBranching {
    GameOver(MCBackprobagation),
    Unexpanded,
    Expanded(Vec<MCNode>),
}

#[derive(Clone, Copy)]
enum MCBackprobagation {
    Victory,
    Loss
}

impl MCNode {
    fn new(state : GameState) -> MCNode {
        MCNode {
            state : state,
            children : MCBranching::Unexpanded,
            play_count : 0,
            victory_count : 0,
        }
    }
}

// fn mc_step(node : &mut MCNode) -> MCBackprobagation {
//     match node.children {
//         // This path has been fully explored
//         MCBranching::GameOver(v) => return v.clone(),
//         MCBranching::Unexpanded => node.children = MCBranching::Expanded(full_expansion(&node.state)),
//         MCBranching::Expanded(children) => return mc_step(choose_random_mut(&mut children)),
//     }
//     panic!();
// }

fn choose_mutable<T>(vec: &mut Vec<T>) -> &mut T {
    let id = rand::thread_rng().gen::<usize>() % vec.len() as usize;

    return &mut vec[id];
}

fn full_expansion(state : &GameState) -> Vec<MCNode> {
    panic!("");
}

// Implementing a min-max tree as well as framework for scoring functions.
// Later..

// pub enum MinMaxTree {
//     // M is the type of the move, T is the type of the gamestate.
//     Unexpanded(GameState), // A gamestate is stored, but the game isn't over yet.
//     Branching(Vec<(Move, MinMaxTree)>),
//     GameOver(GameState), // A gamestate is stored and the game is over.
//     Scored { score : f32, content : MinMaxTree},
// }
//
// impl MinMaxTree {
//     pub fn new(root : GameState) -> MinMaxTree {
//         MinMaxTree::Unexpanded(root)
//     }
//     pub fn expand(&mut self, depth : i8, expander : extern fn (GameState)){
//         match self {
//             Unexpanded(state) =>
//             GameOver(_) => return,
//         }
//     }
//     // pub fn min_max_decision(&mut self, depth : i8,
//     //     expander         : extern fn(GameState) -> Option<Vec<GameState>>,
//     //     scoring_function : extern fn(GameState) -> f32)
//     //         -> Move {
//     //     if depth > 0 {
//     //
//     //     } else {
//     //         // Score the current
//     //     }
//     // }
// }
