// This file contains basic definitions for AIs as well as helper function to build them.
// Some example AIs are also contained.
extern crate rand;
use self::rand::{thread_rng, Rng};
use game;
use game::{GameState, GameStructure, PlayerColor, VictoryState, VictoryStats, LineState, Move};

pub trait SogoAI {
    fn reset_game(&self);
    // Some information may be preserved after an opponent's turn.
    // Tree based algorithms may carry over part of the search tree.
    fn register_opponent_action(&self, &Move);
    fn decide_action(&self, state : &GameState) -> Move;
        // An imutable reference to the game_state is passed for convenience only.
}

pub fn run_match<T : SogoAI, U : SogoAI>(structure : &GameStructure, white_player : &T, black_player : &U) -> GameState {
    let mut i = 0;

    let mut state = GameState::new();
    while state.victory_state == VictoryState::Undecided {
        if state.age == 64 {
            state.victory_state = VictoryState::Draw;
            return state;
        }
        let action = if i % 2 == 0 {white_player.decide_action(&state)} else {black_player.decide_action(&state)};
        if i % 2 == 0 {black_player.register_opponent_action(&action)} else {white_player.register_opponent_action(&action)};
        game::execute_move(structure, &mut state, action);
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

#[allow(dead_code)]
impl RandomSogoAI {
    pub fn new() -> RandomSogoAI {
        RandomSogoAI { alibi : 42 }
    }
}

impl SogoAI for RandomSogoAI {
    fn reset_game(&self) { }
    fn register_opponent_action(&self, _ : &Move) {}
    fn decide_action(&self, state : &GameState) -> Move {
        let position = thread_rng().choose(&state.legal_moves);
        // Rust also implements a faster random generator, but it needs to be stored outside of this
        // small function. Caching the RNG might help anyways.
        match position {
            Some(&(x, y)) => Move::Play {x:x, y:y},
            None => Move::Surrender
        }
    }
}

pub fn random_playout(structure : &GameStructure, state : &GameState) -> VictoryState {
    let mut my_state = state.clone();
    let mut rng = thread_rng();
    while my_state.victory_state == VictoryState::Undecided {
        let action = {
            match rng.choose(&my_state.legal_moves) {
                Some(&(x, y)) => Move::Play {x:x, y:y},
                None => Move::Surrender
            }
        };
        game::execute_move(structure, &mut my_state, action);
    }
    return my_state.victory_state;
}


pub fn random_playout_sample(structure : &GameStructure, state : &GameState, number : i32) -> VictoryStats {
    let mut statics = game::VictoryStats::new();
    for _ in 0..number {
        let result = random_playout(&structure, &state);
        match result {
            game::VictoryState::Win(game::PlayerColor::White) => statics.white += 1,
            game::VictoryState::Win(game::PlayerColor::Black) => statics.black += 1,
            game::VictoryState::Draw      => statics.draws  += 1,
            game::VictoryState::Undecided => panic!("The game_state should never be undecided after a random playout."),
        }
    }
    return statics
}

fn easy_judgement (state : &GameState, my_color : PlayerColor) -> i32 {
    let mut score = 0;
    for i in 0..76 {
        let line = state.lines[i];
        score += match line {
            LineState::Empty  => 0,
            LineState::Win(color) => 1000 * (if color == my_color {1} else {-1}),
                // If I'm still allowed to play, that must have been my win.
            LineState::Mixed  => 0,
            LineState::Pure { color, count } =>
                (count * count * (if color == my_color {1} else {-1})) as i32,
        }
    }
    return score
}

#[allow(dead_code)]
pub struct TreeJudgementAI {
    structure : game::GameStructure,
    search_depth : i8,
}

#[allow(dead_code)]
impl TreeJudgementAI {
    pub fn new(depth : i8) -> TreeJudgementAI {
        TreeJudgementAI { structure : game::GameStructure::new(), search_depth : depth }
    }
}

impl SogoAI for TreeJudgementAI {
    fn reset_game(&self) {}
    fn register_opponent_action(&self, _ : &Move) {}
    fn decide_action(&self, state : &GameState) -> Move {
        let my_color = state.current_color;
        // Create a tree from the current gamestate.
        let mut tree : Node<MinMaxTagging> = Node::new(state.clone(), None);
        // Completely expand the first n layers
        fully_expand_to_depth(&self.structure, &mut tree, self.search_depth);

        let my_easy_judgement = |state : &GameState|
            MinMaxTagging { value : easy_judgement(state, my_color), from_action : None};

        tag_all_leaves(&my_easy_judgement, &mut tree);
        min_max(&mut tree);

        let action = tree.tag.from_action.unwrap_or(Move::Surrender);
        println!("{:?} deciding on '{:?}' with valuation {:?}.", my_color, action, tree.tag.value);
        return action;
    }
}

// The type parameter T can carry additional information (like a score) and needs to allow a default.
struct Node<T : Default> {
    state : GameState,
    children : Branching<T>,
    parent_action : Option<Move>, // If this isn't the root, how did we get here?
    tag : T,
}

enum Branching<T : Default> {
    GameOver,
    Unexpanded,
    Expanded(Vec<Node<T>>),
}

impl<T : Default> Node<T> {
    pub fn new(state : GameState, parent_action : Option<Move>) -> Node<T> {
        Node {
            state : state,
            children : Branching::Unexpanded,
            parent_action : parent_action,
            tag : Default::default(),
        }
    }
}

// Expands the current supplied node, if neccessary. The return value indicates if it was expanded.
fn expand_node_total<T : Default>(structure : &GameStructure, node : &mut Node<T>) -> bool {
    match node.children {
        Branching::Unexpanded => {
            let mut children = Vec::new();
            for action in &node.state.legal_moves {
                let play = Move::new(action);
                let mut child = Node::new(
                    game::execute_move_functional(structure, &node.state, play.clone()),
                    Some(play)
                );
                if child.state.victory_state != VictoryState::Undecided {
                    child.children = Branching::GameOver;
                }
                children.push(child);
            }
            if children.len() == 0 {
                node.children = Branching::GameOver;
            } else {
                node.children = Branching::Expanded(children);
            }
            true
        },
        Branching::Expanded(_) => false,
        Branching::GameOver => false,
    }
}

fn fully_expand_to_depth<T : Default>(structure : &GameStructure, node : &mut Node<T>, depth : i8) {
    if depth <= 0 {
        return;
    }
    expand_node_total(structure, node);
    match node.children {
        Branching::Expanded(ref mut children) => {
            for mut child in children {
                fully_expand_to_depth(structure, &mut child, depth-1);
            }
        },
        _ => (),
    }
}

fn tag_all_leaves<T : Default, F>(tagger : &F, node : &mut Node<T>)
    where F : Fn(&GameState) -> T {
    match node.children {
        Branching::GameOver   => node.tag = tagger(&node.state),
        Branching::Unexpanded => node.tag = tagger(&node.state),
        Branching::Expanded(ref mut children) => {
            for mut child in children {
                tag_all_leaves(tagger, &mut child);
            }
        }
    }
}

#[derive(Default, Debug)]
struct MinMaxTagging {
    value : i32,
    from_action : Option<Move>,
}

fn min_max(node : &mut Node<MinMaxTagging>) {
    match node.children {
        Branching::Expanded(ref mut children) => {
            let mut max = i32::min_value();
            let mut action = None;
            for mut child in children {
                max_min(&mut child);
                if child.tag.value > max {
                    max = child.tag.value;
                    action = child.parent_action;
                }
            }
            node.tag.value = max;
            node.tag.from_action = action;
        },
        _ => () // Leaves don't min-max
    }
}

fn max_min(node : &mut Node<MinMaxTagging>) {
    match node.children {
        Branching::Expanded(ref mut children) => {
            let mut min = i32::max_value();
            let mut action = None;
            for mut child in children {
                min_max(&mut child);
                if child.tag.value < min {
                    min = child.tag.value;
                    action = child.parent_action;
                }
            }
            node.tag.value = min;
            node.tag.from_action = action;
        },
        _ => () // Leaves don't min-max
    }
}

// Pure Monte Carlo AI
// For each possible move, a number of playouts is run. This should give an approximate information
// about the value of each move.

#[allow(dead_code)]
pub struct MonteCarloAI {
    endurance : i32, // How many random games am I allowed to play each turn?
    structure : GameStructure,
}

#[allow(dead_code)]
impl MonteCarloAI {
    pub fn new(endurance : i32) -> MonteCarloAI {
        MonteCarloAI{endurance : endurance, structure : GameStructure::new()}
    }
}

impl SogoAI for MonteCarloAI {
    fn reset_game(&self) {}
    // Some information may be preserved after an opponent's turn.
    // Tree based algorithms may carry over part of the search tree.
    fn register_opponent_action(&self, _ : &Move) {}
    fn decide_action(&self, state : &GameState) -> Move {
        let my_color = state.current_color;
        let endurance_per_action = self.endurance / (state.legal_moves.len() as i32);
        // Create a tree from the current gamestate.
        let mut tree : Node<MinMaxTagging> = Node::new(state.clone(), None);
        // Completely expand the first layer
        fully_expand_to_depth(&self.structure, &mut tree, 1);

        let judgement = |state : &GameState|
            MinMaxTagging {
                value : monte_carlo_judgement(&self.structure, state, my_color, endurance_per_action),
                from_action : None
            };

        tag_all_leaves(&judgement, &mut tree);
        min_max(&mut tree);

        let action = tree.tag.from_action.unwrap_or(Move::Surrender);
        //println!("{:?} deciding on '{:?}' with valuation {:?}.", my_color, action, tree.tag.value);
        return action;
    }
}

fn monte_carlo_judgement(structure : &GameStructure, state : &GameState, my_color : PlayerColor, amount : i32) -> i32 {
    let stats = random_playout_sample(structure, state, amount);
    if my_color == PlayerColor::White {
        return stats.white - stats.black;
    } else {
        return stats.black - stats.white;
    }
}

// Monte Carlo Tree search
// This is fancy, I'll do it later, when I learned about Monte Carlo and
// about Tree search.
