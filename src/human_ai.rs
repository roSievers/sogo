
use std::io;
use game;
use game::{GameState, PointState, PlayerColor, Action};
use ai::{SogoAI};

#[allow(dead_code)] // Sometimes this AI may not be in use, this is why I allow dead code.
pub enum HumanPlayer {
    Active
}

pub fn print_gamestate(state : &GameState) {
    // Here I am iterating y, z, x in that strange order to get a nice output.
    println!("  ----   ----   ----   ---- ");
    for y in (0..4).rev() {
        let mut line = "".to_string();
        for z in 0..4 {
            line = line + " |";
            for x in 0..4 {
                let flat_coordinate = game::flatten(x, y, z);
                line = line + match state.points[flat_coordinate as usize] {
                    PointState::Empty => ".",
                    PointState::Piece(PlayerColor::White) => "X",
                    PointState::Piece(PlayerColor::Black) => "O"
                }
            }
            line = line + "|";
        }
        println!("{}", line);
    }
    println!("  ----   ----   ----   ---- ");
}

pub fn ask_for_move() -> Action {
    let mut instruction = String::new();

    io::stdin().read_line(&mut instruction).expect("Failed to read line");

    let s = String::from("0123456789ABCDEF");
    let _index = s.find(instruction.chars().nth(0).unwrap());

    let index = match _index {
        None    => return Action::Surrender,
        Some(i) => i as i8
    };

    return Action::new(index % 4, index / 4);
}

impl SogoAI for HumanPlayer {
    fn reset_game(&self) {}
    fn register_opponent_action(&self, action : &Action) {
        println!("Enemy action was: {:?}", action);
    }
    fn decide_action(&self, state : &game::GameState) -> Action {
        print_gamestate(state);
        ask_for_move()
    }
}
