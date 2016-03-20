use std::ops::{Not};

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum PlayerColor {
    White,
    Black
}

// This implementation allows PlayerColor::White == !PlayerColor::Black.
impl Not for PlayerColor {
    type Output = PlayerColor;

    fn not(self) -> PlayerColor {
        match self {
            PlayerColor::White => PlayerColor::Black,
            PlayerColor::Black => PlayerColor::White
        }
    }
}

#[allow(dead_code)] // Dead code is allowed, because I want x, y, z even if I don't use it.
#[derive(Debug, Clone)]
pub struct Point {
    pub x : i8,
    pub y : i8,
    pub z : i8,
    pub flat_coordinate : i8,
    pub lines : Vec<i8> // This vector stores the IDs of all lines through it.
}

impl Point {
    // constructor, by convention
    pub fn new(x:i8, y:i8, z:i8) -> Point {
        Point {x:x, y:y, z:z, flat_coordinate:flatten(x, y, z), lines:Vec::new()} //, lines : Vec::new()}
    }
}

pub fn flatten(x:i8, y:i8, z:i8) -> i8 {
    return x + 4*y + 16*z
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Action {
    Play {x : i8, y : i8},
    Surrender
}

impl Action {
    pub fn new(x : i8, y : i8) -> Action {
        Action::Play {x:x, y:y}
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum PointState {
    Piece(PlayerColor),
    Empty
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum LineState {
    Empty,
    Pure { color: PlayerColor, count: i8 },
    Mixed,
    Win(PlayerColor),
}

impl LineState {
    fn add_ball_functional(line_state : LineState, new_color : PlayerColor) -> LineState {
        match line_state {
            LineState::Empty => LineState::Pure { color : new_color, count : 1},
            LineState::Pure { color : current_color, count : old_count} =>
                if current_color != new_color { LineState::Mixed } else {
                    if old_count == 3 {LineState::Win(current_color)}
                    else {LineState::Pure {color : current_color, count : old_count+1}}
                },
            LineState::Mixed => LineState::Mixed,
            LineState::Win(_) => panic!("A filled line can't accept any more balls.")
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum VictoryState {
    Undecided,
    Win(PlayerColor),
    Draw
}

#[derive(Debug, Copy, Clone)]
pub struct VictoryStats {
    pub white : i32,
    pub black : i32,
    pub draws : i32,
}

impl VictoryStats {
    pub fn new() -> VictoryStats {
        VictoryStats { white : 0, black : 0, draws : 0}
    }
}

#[derive(Clone)]
pub struct GameStructure {
    pub points : Vec<Point>,
    victory_object_count : usize,
}

impl GameStructure {
    pub fn new(victory_objects : &[u64]) -> GameStructure {
        // Initialize a vector of all Points.
        let mut point_box = Vec::new();
        for z in 0..4 {
            for y in 0..4 {
                for x in 0..4 {
                    point_box.push(Point::new(x, y, z));
                }
            }
        }

        // Refenence the line ID in the points.
        for line_id in 0..victory_objects.len() {
            // rep is a u64 encoding of the line.
            let mut rep = victory_objects[line_id];
            let mut flat = 0;
            while rep > 0 {
                // A one indicates that this line has a point at that particular position.
                if rep % 2 == 1 {
                    point_box[flat as usize].lines.push(line_id as i8);
                }
                rep /= 2;
                flat += 1;
            }
        }

        GameStructure { points : point_box, victory_object_count : victory_objects.len() }
    }
}

#[derive(Clone)]
pub struct GameState {
    pub points : Vec<PointState>, // Maybe we want to change this into a Vector.
    pub lines  : Vec<LineState>,
    pub current_color : PlayerColor,
    pub victory_state : VictoryState,
    pub age : i8, // how many balls were played?
    pub legal_actions : Vec<Action>,
}

impl GameState {
    pub fn new(structure : &GameStructure) -> GameState {
        // The board is empty and all actions are legal
        let mut legal = Vec::new();
        for x in 0..4 {
            for y in 0..4 {
                legal.push(Action::new(x,y));
            }
        }
        GameState {
            points : vec![PointState::Empty; 64],
            lines : vec![LineState::Empty; structure.victory_object_count],
            current_color : PlayerColor::White,
            victory_state : VictoryState::Undecided,
            age : 0,
            legal_actions : legal,
        }
    }

    pub fn execute_action(&mut self, structure : &GameStructure, play : &Action) {
        match play {
            &Action::Surrender   => self.victory_state = VictoryState::Win(!self.current_color),
            &Action::Play {x, y} => self.play_at(structure, x, y),
        }
    }
    pub fn execute_action_functional(&self, structure : &GameStructure, play : &Action) -> GameState {
        let mut result = self.clone();
        result.execute_action(structure, &play);
        return result;
    }

    // Idea: This could be cached inside each gamestate.
    // Height at which a new ball would be placed.
    fn z_value(&self, x : i8, y : i8) -> Option<i8> {
        for z in 0..4 {
            if self.points[flatten(x, y, z) as usize] == PointState::Empty {
                return Some(z)
            }
        }
        return None
    }

    fn play_at(&mut self, structure : &GameStructure, x:i8, y:i8) {
        let z = self.z_value(x, y);
        let flat_coordinate = match z {
            Some(z) => flatten(x, y, z),
            None => panic!("Added a ball at ({}, {}), which is already full.", x, y)
        };
        // Place a colored piece at the coordinate
        self.points[flat_coordinate as usize] = PointState::Piece(self.current_color);
        // Update the legal actions, if the z-coordinate is 3
        // I make use of the fact that the z coordinate is occupying the top two bits.
        if flat_coordinate >= 4*4*3 {
            // As the legal actions will be mixed up during play, we need to search through all.
            for i in 0..self.legal_actions.len() {
                // TODO: Don't construct a new Action object just to compare.
                if self.legal_actions[i] == Action::new(x, y) {
                    self.legal_actions.swap_remove(i);
                    // Removes an element from anywhere in the vector and return it, replacing it with the last element.
                    // This does not preserve ordering, but is O(1).
                    break;
                }
            }
        }
        for line in structure.points[flat_coordinate as usize].lines.clone() {
            let line_state = LineState::add_ball_functional(self.lines[line as usize], self.current_color);
            match line_state {
                LineState::Win(color) => self.victory_state = VictoryState::Win(color),
                _ => (),
            }
            self.lines[line as usize] = line_state;
        }
        self.age += 1;
        if self.age == 64 && self.victory_state == VictoryState::Undecided {
            self.victory_state = VictoryState::Draw;
        }
        self.current_color = !self.current_color;
    }
}
