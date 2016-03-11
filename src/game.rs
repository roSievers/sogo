
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
#[derive(Debug)]
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

pub struct Line {
    points : [Point; 4]
}

impl Line {
    // constructor, by convention
    pub fn new(x:i8, y:i8, z:i8, dx:i8, dy:i8, dz:i8) -> Line {
        let point1 = Point::new(x, y, z);
        let point2 = Point::new(x+1*dx, y+1*dy, z+1*dz);
        let point3 = Point::new(x+2*dx, y+2*dy, z+2*dz);
        let point4 = Point::new(x+3*dx, y+3*dy, z+3*dz);
        Line { points : [point1, point2, point3, point4]}
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

// TODO: Recomputing this and passing it around gets boring at some point.
// Once we abstract over several different game structures
// this should be moved to a macro and be created once at compile time.
pub struct GameStructure {
    pub points : Vec<Point>,
    pub lines  : Vec<Line>
}

impl GameStructure {
    pub fn new() -> GameStructure {
        // Initialize a vector of all Points.
        let mut point_box = Vec::new();
        for z in 0..4 {
            for y in 0..4 {
                for x in 0..4 {
                    point_box.push(Point::new(x, y, z));
                }
            }
        }

        // Initialize a vector of all Lines.
        let mut line_box = Vec::new();
        for a in 0..4 {
            for b in 0..4 {
                line_box.push(Line::new(a, b, 0, 0, 0, 1));
                line_box.push(Line::new(0, a, b, 1, 0, 0));
                line_box.push(Line::new(b, 0, a, 0, 1, 0))
            }
            // Diagonals in two spacial directions
            line_box.push(Line::new(a, 0, 0, 0, 1, 1));
            line_box.push(Line::new(0, a, 0, 1, 0, 1));
            line_box.push(Line::new(0, 0, a, 1, 1, 0));
            line_box.push(Line::new(a, 3, 0, 0,-1, 1));
            line_box.push(Line::new(0, a, 3, 1, 0,-1));
            line_box.push(Line::new(3, 0, a,-1, 1, 0));
        }
        // Diagonals in all three directions at once
        line_box.push(Line::new(0, 0, 0, 1, 1, 1));
        line_box.push(Line::new(3, 0, 0,-1, 1, 1));
        line_box.push(Line::new(3, 3, 0,-1,-1, 1));
        line_box.push(Line::new(0, 3, 0, 1,-1, 1));

        // Refenence the line ID in the points.
        let mut i = 0;
        for line in &line_box {
            for point in line.points.iter() {
                point_box[point.flat_coordinate as usize].lines.push(i);
            }
            i += 1;
        }
        GameStructure { points : point_box, lines : line_box }
    }
}

#[test]
fn test_game_structure_size() {
    let structure = GameStructure::new();
    assert_eq!(structure.points.len(), 4*4*4);
    let mut i = 0;
    for p in &structure.points {
        assert_eq!(p.flat_coordinate, i);
        i += 1;
    }
    assert_eq!(structure.lines.len(), 76);
}

// fn expand(index:i8) -> (i8, i8, i8) {
//     return (index % 4, (index / 4) % 4, index / 16)
// }



//#[derive(Clone)]
pub struct GameState {
    pub points : [PointState; 64], // Maybe we want to change this into a Vector.
    pub lines  : [LineState; 76],
    pub current_color : PlayerColor,
    pub victory_state : VictoryState,
    pub age : i8, // how many balls were played?
    pub legal_actions : Vec<Action>,
}

impl Clone for GameState {
    fn clone(&self) -> GameState {
        let mut point_clone = [PointState::Empty; 64];
        let mut line_clone  = [LineState::Empty; 76];
        for i in 0..64 {
            point_clone[i] = self.points[i].clone();
        }
        for i in 0..76 {
            line_clone[i] = self.lines[i].clone();
        }
        GameState {
            points : point_clone,
            lines  : line_clone,
            current_color : self.current_color.clone(),
            victory_state : self.victory_state.clone(),
            age : self.age.clone(),
            legal_actions : self.legal_actions.clone(),
        }
    }
}

impl GameState {
    pub fn new() -> GameState {
        // The board is empty and all actions are legal
        let mut legal = Vec::new();
        for x in 0..4 {
            for y in 0..4 {
                legal.push(Action::new(x,y));
            }
        }
        GameState {
            points : [PointState::Empty; 64],
            lines : [LineState::Empty; 76],
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
                    println!("removed something.");
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
