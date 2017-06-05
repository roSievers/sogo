
use ai::StatelessAI;

use game;
use game::{Action, GameStructure, PlayerColor, LineState, VictoryState};
use std::rc::Rc;


fn easy_judgement(structure: &GameStructure, state: &game::State, my_color: PlayerColor) -> i32 {
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
        // Create a tree from the current gamestate.
        let mut tree: Node<MinMaxTagging> = Node::new(state.clone(), None);
        // Completely expand the first n layers
        tree.fully_expand_to_depth(&self.structure, self.search_depth);

        let my_easy_judgement = |state: &game::State| {
            MinMaxTagging {
                value: easy_judgement(&self.structure, state, my_color),
                from_action: None,
            }
        };

        tree.tag_all_leaves(&my_easy_judgement);
        min_max(&mut tree);

        let action = tree.tag.from_action.unwrap_or(Action::Surrender);
        println!("{:?} deciding on '{:?}' with valuation {:?}.",
                 my_color,
                 action,
                 tree.tag.value);
        return action;
    }
}

// The type parameter T can carry additional information (like a score)
// and needs to allow a default.
struct Node<T: Default> {
    state: game::State,
    children: Branching<T>,
    parent_action: Option<Action>, // If this isn't the root, how did we get here?
    tag: T,
}

enum Branching<T: Default> {
    GameOver,
    Unexpanded,
    Expanded(Vec<Node<T>>),
}

impl<T: Default> Node<T> {
    pub fn new(state: game::State, parent_action: Option<Action>) -> Node<T> {
        Node {
            state: state,
            children: Branching::Unexpanded,
            parent_action: parent_action,
            tag: Default::default(),
        }
    }

    // Expands the current supplied node, if neccessary.
    // The return value indicates if it was expanded.
    fn expand_total(&mut self, structure: &GameStructure) -> bool {
        match self.children {
            Branching::Unexpanded => {
                let mut children = Vec::new();
                for action in self.state.legal_actions() {
                    let mut new_state = self.state.clone();
                    new_state.execute(structure, action);
                    let mut child =
                        Node::new(new_state,
                                  Some(action.clone()));
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
            }
            Branching::Expanded(_) => false,
            Branching::GameOver => false,
        }
    }

    fn fully_expand_to_depth(&mut self, structure: &GameStructure, depth: i8) {
        if depth <= 0 {
            return;
        }
        self.expand_total(structure);
        match self.children {
            Branching::Expanded(ref mut children) => {
                for mut child in children {
                    child.fully_expand_to_depth(structure, depth - 1);
                }
            }
            _ => (),
        }
    }

    fn tag_all_leaves<F>(&mut self, tagger: &F)
        where F: Fn(&game::State) -> T
    {
        match self.children {
            Branching::GameOver => self.tag = tagger(&self.state),
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
    value: i32,
    from_action: Option<Action>,
}

fn min_max(node: &mut Node<MinMaxTagging>) {
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
        }
        _ => (), // Leaves don't min-max
    }
}

fn max_min(node: &mut Node<MinMaxTagging>) {
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
        }
        _ => (), // Leaves don't min-max
    }
}
