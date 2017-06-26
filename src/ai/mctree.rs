/* Implements monte carlo tree search

From PROGRESSIVE STRATEGIES FOR MONTE-CARLO TREE SEARCH,
which was linked in the Wikipedia.
(https://dke.maastrichtuniversity.nl/m.winands/documents/pMCTS.pdf)

Selection: start from root R and select successive child nodes down to a leaf node L.
Expansion: unless L ends the game with a win/loss for either player, create child nodes.
Simulation: play a random playout from node C.
Backpropagation: use the result of the playout to update information in the nodes.

*/

use rand::{thread_rng, Rng};

use ai::StatelessAI;
use game;
use game::Position2;

pub struct MCTreeAI {
    endurance: usize,
    exploration: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Index(usize);

// Using a VecTree strategy to sidestep the borrow checker.
// Doing anything tree based, especially with back references to parents
// always involves struggling against the borrowck.
// This is basically implementing pointers by hand. But propper
// livetime checking or garbage collection is only necessary when
// objects ever get deleted. That doesn't happen here.
struct VecTree {
    storage: Vec<Node>,
}

impl VecTree {
    fn new(capacity: usize, state: &game::State) -> Self {
        let mut storage = Vec::with_capacity(capacity);
        storage.push(Node::new(None, state));
        VecTree { storage }
    }
    fn robust_move(&self) -> Position2 {
        // Select the action, which was played most often. Apparently this
        // is more robust than using the action with the best win ratio.
        let ref root = self.storage[0];

        let mut most_robust = vec![];
        let mut most_simulations = 0;

        for i in 0..16 {
            match root.children[i] {
                ChildRef::Expanded(child_index) => {
                    let ref child = self.storage[child_index.0];
                    if child.simulation_count > most_simulations {
                        most_robust = vec![i];
                        most_simulations = child.simulation_count;
                    } else if child.simulation_count == most_simulations {
                        most_robust.push(i);
                    } else {
                        // This child is worse than a previously seen child.
                    }
                }
                _ => {}
            }
        }

        let choosen_position = *thread_rng().choose(&most_robust).unwrap();

        // Finally, we got the best move - return it to play it.
        Position2(choosen_position as u8)
    }
}

struct Node {
    // The win_count counts wins - losses and can be negative.
    win_count: isize,
    simulation_count: usize,

    parent: Option<Index>,
    children: [ChildRef; 16],
}

impl Node {
    fn new(parent_index: Option<Index>, state: &game::State) -> Self {
        let mut children = [ChildRef::NotYetExpanded; 16];
        for i in 0..16 {
            if state.column_height[i] == 4 {
                children[i] = ChildRef::IllegalMove;
            }
        }
        Node {
            win_count: 0,
            simulation_count: 0,
            parent: parent_index,
            children,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ChildRef {
    IllegalMove,
    NotYetExpanded,
    Expanded(Index),
}

impl VecTree {
    fn select_best(
        &mut self,
        node_index: Index,
        mut state: game::State,
        exploration: f32,
    ) -> (Index, game::State) {
        use std::f32;

        if !state.victory_state.active() {
            return (node_index, state);
        }

        let mut candidates: Vec<usize> = vec![];
        let mut best_value: f32 = f32::NEG_INFINITY;

        for i in 0..16 {
            match self.storage[node_index.0].children[i] {
                ChildRef::IllegalMove => {}
                ChildRef::NotYetExpanded => {
                    return self.initialize_random_child(node_index, i, state);
                }
                ChildRef::Expanded(child_index) => {
                    let ref node = self.storage[node_index.0];
                    let ref child = self.storage[child_index.0];

                    let exploitation_value = child.win_count as f32 / child.simulation_count as f32;
                    let exploration_value = exploration *
                        f32::sqrt(
                            f32::ln(node.simulation_count as f32) / child.simulation_count as f32,
                        );

                    let value = exploration_value + exploitation_value;

                    if value > best_value {
                        best_value = value;
                        candidates = vec![i];
                    } else if value == best_value {
                        candidates.push(i);
                    } else {
                        // Do nothing, this child doesn't qualify for the random draw.
                    }
                }
            }
        }

        // TODO: We can only sensibly unwrap, if we make sure that
        // the game is still ongoing at this moment.
        let choosen_position = *thread_rng().choose(&candidates).unwrap_or_else(|| {
            panic!("Trying to choose a best child when no children are avaliable.")
        });

        state.execute(Position2(choosen_position as u8));

        if let ChildRef::Expanded(child_index) =
            self.storage[node_index.0].children[choosen_position]
        {

            self.select_best(child_index, state, exploration)

        } else {
            // The choosen_position is drawn from indices
            // where an expanded child is guaranteed to exist.
            panic!("The choosen child somehow isn't expanded.");
        }
    }
    // Selects and initializes an uninitialized move.
    fn initialize_random_child(
        &mut self,
        node_index: Index,
        first_uninitialized_child_index: usize,
        mut state: game::State,
    ) -> (Index, game::State) {
        let new_index = Index(self.storage.len());

        let choosen_position = {
            // To satisfy borrowck, I first mutate the parent node itself,
            // then drop the mutable reference to it and add the new child
            // to the vector which also contains the node.
            // Borrowck rightfully complains about this, because adding a child
            // might reallocate the whole vector, rendering the node pointer invalid.
            let ref mut node = self.storage[node_index.0];

            let mut uninitialized_children = vec![first_uninitialized_child_index];
            for i in (first_uninitialized_child_index + 1)..16 {
                if node.children[i] == ChildRef::NotYetExpanded {
                    uninitialized_children.push(i);
                }
            }

            let choosen_position = *thread_rng().choose(&uninitialized_children).unwrap();

            node.children[choosen_position] = ChildRef::Expanded(new_index);
            choosen_position
        };

        state.execute(Position2(choosen_position as u8));
        self.storage.push(Node::new(Some(node_index), &state));

        (new_index, state)
    }
    fn backpropagate(&mut self, node_index: Index, value: isize) {
        if let Some(parent_index) =
            {
                // This is wrapped in { .. } in order to drop the mutable reference
                // to the node before using &mut self again.
                let ref mut node = self.storage[node_index.0];
                node.win_count += value;
                node.simulation_count += 1;
                node.parent
            }
        {
            self.backpropagate(parent_index, -value);
        }

    }
}


impl MCTreeAI {
    pub fn new(endurance: usize, exploration: f32) -> Self {
        MCTreeAI {
            endurance,
            exploration,
        }
    }
    fn create_tree_async(&self, state: &game::State) -> VecTree {
        use ai::mc::random_playout;
        use threadpool::ThreadPool;
        use std::sync::mpsc::channel;

        // Set up everything async.
        let worker_count = 4;
        let pool = ThreadPool::new(worker_count);
        let (sender, receiver) = channel();

        // Set up an empty tree.
        let mut tree = VecTree::new(self.endurance + 1, state);
        assert!(
            self.endurance > worker_count,
            "The endurance must be larger than {}.",
            worker_count
        );
        let mut endurance_left = self.endurance - worker_count;

        // Create the first few packages worth of work.
        for _ in 0..worker_count {
            // Selection & Expansion
            let (leaf_index, leaf_state) =
                tree.select_best(Index(0), state.clone(), self.exploration);

            // Simulation
            let sender_clone = sender.clone();
            pool.execute(move || {
                let current_color = leaf_state.current_color;
                let score = random_playout(leaf_state).scoring(current_color).unwrap() as isize;

                sender_clone.send((leaf_index, score)).unwrap();
            });
        }

        // Now wait for results
        while let Ok((leaf_index, score)) = receiver.recv() {
            // Backpropagation
            tree.backpropagate(leaf_index, -score);

            if endurance_left > 0 {
                endurance_left -= 1;
                // Generate new work
                // Selection & Expansion
                let (leaf_index, leaf_state) =
                    tree.select_best(Index(0), state.clone(), self.exploration);

                // Simulation
                let sender_clone = sender.clone();
                pool.execute(move || {
                    let current_color = leaf_state.current_color;
                    let score = random_playout(leaf_state).scoring(current_color).unwrap() as isize;

                    sender_clone.send((leaf_index, score)).unwrap();
                });
            } else if pool.active_count() == 0 {
                break;
            }
        }

        tree
    }
}


impl StatelessAI for MCTreeAI {
    fn action(&self, state: &game::State) -> Position2 {
        self.create_tree_async(state).robust_move()
    }
}
