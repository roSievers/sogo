// This file contains basic definitions for AIs as well as helper function to build them.
// Some example AIs are also contained.
extern crate rand;
use self::rand::{thread_rng, Rng, Rand};
//use self::rand::distributions::{IndependentSample, Range};
use game;
use game::{GameState, GameStructure, PlayerColor, VictoryState, VictoryStats, LineState, Action};
use std::rc::Rc;
use helpers::upper_bound_index;

pub trait SogoAI {
    fn reset_game(&mut self);
    // Some information may be preserved after an opponent's turn.
    // Tree based algorithms may carry over part of the search tree.
    fn register_opponent_action(&mut self, &Action);
    fn decide_action(&mut self, state : &GameState) -> Action;
        // An imutable reference to the game_state is passed for convenience only.
}

pub fn run_match<T : SogoAI, U : SogoAI>(
        structure : &GameStructure, white_player : &mut T, black_player : &mut U)
        -> GameState {
    let mut i = 0;

    let mut state = GameState::new(&structure);
    while state.victory_state == VictoryState::Undecided {
        if state.age == 64 {
            state.victory_state = VictoryState::Draw;
            return state;
        }
        let action = if i % 2 == 0 {white_player.decide_action(&state)} else {black_player.decide_action(&state)};
        if i % 2 == 0 {black_player.register_opponent_action(&action)} else {white_player.register_opponent_action(&action)};
        state.execute_action(structure, &action);
        i += 1;
    }
    // println!("{:?}", i);
    return state;
}

// An AI which executes random legal actions
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
    fn reset_game(&mut self) { }
    fn register_opponent_action(&mut self, _ : &Action) {}
    fn decide_action(&mut self, state : &GameState) -> Action {
        thread_rng().choose(&state.legal_actions)
            .map_or(Action::Surrender, |&a| a.clone())
        // Rust also implements a faster random generator, but it needs to be stored outside of this
        // small function. Caching the RNG might help anyways.
    }
}

pub fn random_playout(structure : &GameStructure, state : &GameState) -> VictoryState {
    let mut my_state = state.clone();
    let mut rng = thread_rng();
    while my_state.victory_state == VictoryState::Undecided {
        let surrender = Action::Surrender;
        let action = rng.choose(&my_state.legal_actions)
                        .unwrap_or(&surrender).clone();
        my_state.execute_action(structure, &action);
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
    for i in 0..state.lines.len() {
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
    structure : Rc<game::GameStructure>,
    search_depth : i8,
}

#[allow(dead_code)]
impl TreeJudgementAI {
    pub fn new(structure : Rc<GameStructure>, depth : i8) -> TreeJudgementAI {
        TreeJudgementAI { structure : structure, search_depth : depth }
    }
}

impl SogoAI for TreeJudgementAI {
    fn reset_game(&mut self) {}
    fn register_opponent_action(&mut self, _ : &Action) {}
    fn decide_action(&mut self, state : &GameState) -> Action {
        let my_color = state.current_color;
        // Create a tree from the current gamestate.
        let mut tree : Node<MinMaxTagging> = Node::new(state.clone(), None);
        // Completely expand the first n layers
        tree.fully_expand_to_depth(&self.structure, self.search_depth);

        let my_easy_judgement = |state : &GameState|
            MinMaxTagging { value : easy_judgement(state, my_color), from_action : None};

        tree.tag_all_leaves(&my_easy_judgement);
        min_max(&mut tree);

        let action = tree.tag.from_action.unwrap_or(Action::Surrender);
        println!("{:?} deciding on '{:?}' with valuation {:?}.", my_color, action, tree.tag.value);
        return action;
    }
}

// The type parameter T can carry additional information (like a score) and needs to allow a default.
struct Node<T : Default> {
    state : GameState,
    children : Branching<T>,
    parent_action : Option<Action>, // If this isn't the root, how did we get here?
    tag : T,
}

enum Branching<T : Default> {
    GameOver,
    Unexpanded,
    Expanded(Vec<Node<T>>),
}

impl<T : Default> Node<T> {
    pub fn new(state : GameState, parent_action : Option<Action>) -> Node<T> {
        Node {
            state : state,
            children : Branching::Unexpanded,
            parent_action : parent_action,
            tag : Default::default(),
        }
    }

    // Expands the current supplied node, if neccessary. The return value indicates if it was expanded.
    fn expand_total(&mut self, structure : &GameStructure) -> bool {
        match self.children {
            Branching::Unexpanded => {
                let mut children = Vec::new();
                for action in &self.state.legal_actions {
                    let mut child = Node::new(
                        self.state.execute_action_functional(structure, &action),
                        Some(action.clone())
                    );
                    if child.state.victory_state != VictoryState::Undecided {
                        child.children = Branching::GameOver;
                    }
                    children.push(child);
                }
                if children.len() == 0 {
                    self.children = Branching::GameOver;
                } else {
                    self.children = Branching::Expanded(children);
                }
                true
            },
            Branching::Expanded(_) => false,
            Branching::GameOver => false,
        }
    }

    fn fully_expand_to_depth(&mut self, structure : &GameStructure, depth : i8) {
        if depth <= 0 {
            return;
        }
        self.expand_total(structure);
        match self.children {
            Branching::Expanded(ref mut children) => {
                for mut child in children {
                    child.fully_expand_to_depth(structure, depth-1);
                }
            },
            _ => (),
        }
    }

    fn tag_all_leaves<F>(&mut self, tagger : &F)
    where F : Fn(&GameState) -> T {
        match self.children {
            Branching::GameOver   => self.tag = tagger(&self.state),
            Branching::Unexpanded => self.tag = tagger(&self.state),
            Branching::Expanded(ref mut children) => {
                for mut child in children {
                    child.tag_all_leaves(tagger);
                }
            }
        }
    }
}


#[derive(Default, Debug)]
struct MinMaxTagging {
    value : i32,
    from_action : Option<Action>,
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
// For each possible action, a number of playouts is run. This should give an approximate information
// about the value of each action.

#[allow(dead_code)]
pub struct MonteCarloAI {
    endurance : i32, // How many random games am I allowed to play each turn?
    structure : Rc<GameStructure>,
}

#[allow(dead_code)]
impl MonteCarloAI {
    pub fn new(structure : Rc<GameStructure>, endurance : i32) -> MonteCarloAI {
        MonteCarloAI{endurance : endurance, structure : structure}
    }
}

impl SogoAI for MonteCarloAI {
    fn reset_game(&mut self) {}
    // Some information may be preserved after an opponent's turn.
    // Tree based algorithms may carry over part of the search tree.
    fn register_opponent_action(&mut self, _ : &Action) {}
    fn decide_action(&mut self, state : &GameState) -> Action {
        let my_color = state.current_color;
        let endurance_per_action = self.endurance / (state.legal_actions.len() as i32);
        // Create a tree from the current gamestate.
        let mut tree : Node<MinMaxTagging> = Node::new(state.clone(), None);
        // Completely expand the first layer
        tree.fully_expand_to_depth(&self.structure, 1);

        let judgement = |state : &GameState|
            MinMaxTagging {
                value : monte_carlo_judgement(&self.structure, state, my_color, endurance_per_action),
                from_action : None
            };

        tree.tag_all_leaves(&judgement);
        min_max(&mut tree);

        let action = tree.tag.from_action.unwrap_or(Action::Surrender);
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

#[derive(Debug)]
pub struct MCNode {
    children : Vec<MCPair>,
    // The victories are from the perspective of the currently active player.
    // This makes it easier to choose a move but this alternation has to be woven into
    // the backpropagation of the victory information.

    // We want to use floats instead of integers, because
    //  a) they are used in float computations most of the time and
    //  b) draws should count as 0.5 victories.
    // But float can not implement Eq, so we need to implement Eq ourself.
    victories : f32,
    total : i32,
}

impl PartialEq<MCNode> for MCNode {
    fn eq(&self, other : &MCNode) -> bool {
        (self.children == other.children) &&
        ((self.victories - other.victories).abs() < 0.4) &&
        (self.total == other.total)
    }
}

impl Eq for MCNode {
    fn assert_receiver_is_total_eq(&self) {}
}

#[derive(Debug, Eq, PartialEq)]
struct MCPair {
    action : Action,
    node : Option<MCNode>,
}

#[allow(dead_code)]
impl MCNode {
    pub fn new(state : &GameState) -> MCNode {
        let mut children = Vec::new();
        for action in &state.legal_actions {
            children.push(MCPair { action : action.clone(), node : None } )
        }
        MCNode { children : children, victories : 0.0, total : 0}
    }
    fn random_child_index(&self, total : i32) -> usize {
        let mut priorities = Vec::new();
        let mut total_priority : f32 = 0.0;
        for index in 0..self.children.len() {
            let node = &self.children[index].node;
            let priority : f32 = match node {
                &None => 2.0, // This is arbitrary right now.
                &Some(ref node_i) => node_i.value(total),
            };
            total_priority += priority;
            priorities.push(total_priority);
        }
        let random_position = f32::rand(&mut thread_rng()) * total_priority;
        return upper_bound_index(&priorities, random_position);
    }
    fn value(&self, total : i32) -> f32 {
        self.victories / self.total as f32 + 1.415 * ((total as f32).ln() / self.total as f32).sqrt()
    }
    // The victory state is returned as a f32 with 0, 0.5, 1 representing loss, draw and victory.
    pub fn random_playout(&mut self, structure : &GameStructure, mut state : GameState, total : i32) -> f32 {
        // First we choose a random child to continue.
        let my_color = state.current_color;
        let child_index = self.random_child_index(total);
        let pair = &mut self.children[child_index];
        state.execute_action(structure, &pair.action);
        let result = match pair.node {
            None => {
                let mut node = MCNode::new(&state);
                // We reached a new leaf. Do one random playout from this position.
                // TODO: This would also benefit from random_playout_ip
                let inner_result = random_playout(structure, &state).as_float(my_color);
                node.total = 1;
                node.victories = inner_result; // 0 + inner_result
                pair.node = Some(node);

                1.0 - inner_result
            },
            Some(ref mut node) => {
                // Recurse into the already existing child.
                1.0 - node.random_playout(structure, state, total)
            }
        };
        self.victories += result;
        self.total += 1;
        return result;
    }
    pub fn print_some_info(&self) {
        println!("There was a total of {} playouts with {} vitories.", self.total, self.victories);

        for index in 0..self.children.len() {
            let node = &self.children[index].node;
            match node {
                &None => {println!("Move {} was never done", index);}
                &Some(ref node_i) => {
                    let priority = node_i.value(self.total);
                    println!("Move {} was won {} out of {} times and has a priority of {}.",
                        index, node_i.victories, node_i.total, priority);
                },
            };
        };
    }
    pub fn get_best_action(&self) -> Action {
        let mut best_action = Action::Surrender;
        let mut largest_total = 0;
        for index in 0..self.children.len() {
            match self.children[index].node {
                None => {},
                Some(ref node) => {
                    let total = node.total;
                    if total > largest_total {
                        largest_total = total;
                        best_action = self.children[index].action;
                    }
                }
            }
        }
        return best_action;
    }
}

pub struct MCTreeAI {
    structure : Rc<GameStructure>,
    endurance : i32,
    state : GameState,
    pub root : MCNode,
}

impl MCTreeAI {
    pub fn new(structure : Rc<GameStructure>, endurance : i32) -> MCTreeAI {
        let state = GameState::new(&structure);
        let root = MCNode::new(&state);
        MCTreeAI {
            structure : structure,
            endurance : endurance,
            state : state,
            root : root,
        }
    }
    pub fn simulate_playout(&mut self) {
        let total = self.root.total;
        self.root.random_playout(&self.structure, self.state.clone(), total);
    }
    fn change_root(&mut self, action : &Action) {
        // The state is updated and the tree is followed downwards.
        // All other pathes are destroyed.
        self.state.execute_action(&self.structure, action);
        let mut new_root = None;
        while let Some(pair) = self.root.children.pop() {
            if pair.action == *action {
                new_root = pair.node;
            }
        }
        self.root = match new_root {
            None        => MCNode::new(&self.state),
            Some(node)  => node,
        };
    }
}

impl SogoAI for MCTreeAI {
    fn reset_game(&mut self) {
        self.state = GameState::new(&self.structure);
        self.root = MCNode::new(&self.state);
    }
    // Some information may be preserved after an opponent's turn.
    // Tree based algorithms may carry over part of the search tree.
    fn register_opponent_action(&mut self, action : &Action) {
        self.change_root(action);
    }
    fn decide_action(&mut self, _ : &GameState) -> Action {
        for _ in 0..self.endurance {
            self.simulate_playout();
        }
        //self.root.print_some_info();
        let action = self.root.get_best_action();
        self.change_root(&action);
        return action;
    }
}

// Monte Carlo Tree search
// This is fancy, I'll do it later, when I learned about Monte Carlo and
// about Tree search.

/*

// This tree structure is slightly more lazy than the Node tree and can expand a node partially.
struct MCNode {
    state : GameState,
    children : MCBranching,
    parent_action : Option<Action>, // If this isn't the root, how did we get here?
    wins : i32,
    total : i32,
}

enum LazyMCNode {
    Unexpanded(Action),
    // We need a Box to avoid [E0072], which forbids nested struct/enums.
    // use 'rustc --explain E0072' to find out more.
    Expanded(Box<MCNode>)
}

// I don't want to evaluate all possible branches to save time and memory.
// Only the possible actions are stored and can be lazily changed to a gamestate.
enum MCBranching {
    GameOver(VictoryState),
    Branch(Vec<LazyMCNode>),
}

impl MCNode {
    pub fn new(state : GameState, parent_action : Option<Action>) -> MCNode {
        let children = {
            if state.victory_state == VictoryState::Undecided {
                let mut children = Vec::new();
                for action in &state.legal_actions {
                    children.push(LazyMCNode::Unexpanded(action.clone()));
                }
                MCBranching::Branch(children)
            } else {
                MCBranching::GameOver(state.victory_state.clone())
            }
        };
        MCNode {
            state : state,
            children : children,
            parent_action : parent_action,
            wins : 0,
            total : 0,
        }
    }
}

fn random_mc_playout(structure : &GameStructure, node : &mut MCNode) -> VictoryState {
    // First, check if we have children.
    match node.children {
        MCBranching::GameOver(victory_state) => victory_state,
        MCBranching::Branch(ref mut children) => {
            // Randomly choose a child
            //let mut child = thread_rng().choose(&children).unwrap();
            // OK, here we have a problem with the saveness of Rust.
            // We can't own children and a child at the same time. This is kind of the reason why
            // choosing randomly can only return a reference to the child.
            // I'll try fixing this by choosing a random index first and then extracting that child.
            // That should give me ownership of a child and suspend children until the reference to
            // child becomes invalid. (At the end of this match block.)
            let between = Range::new(0, children.len());
            let index = between.ind_sample(&mut thread_rng());
            let ref mut child = children[index];
            match child {
                // if this is a unexpanded node, do a random playout, count it and propagate it upwards.
                &mut LazyMCNode::Unexpanded(action) => {
                    let mut new_state = node.state.clone();
                    new_state.execute_action(structure, &action);
                    // This is bad, we clone the state twice.
                    // TODO: Define random_playout_ip to speed this up.
                    let result = random_playout(structure, &mut new_state);
                    result
                }
                &mut LazyMCNode::Expanded(ref mut boxed_node) => {
                    // Randomly choose a child to follow.
                    VictoryState::Draw
                }
            }
            //VictoryState::Draw
        }
    }
}
// */
