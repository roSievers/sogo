
use std::ops::{Not};

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum PlayerColor {
    White,
    Black
}

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

#[derive(Debug)]
pub enum Move {
    Play {x : i8, y : i8},
    Surrender
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

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum VictoryState {
    Undecided,
    Win(PlayerColor),
    Draw
}

//#[derive(Clone)]
pub struct GameState {
    pub points : [PointState; 64],
    pub lines  : [LineState; 76],  // something something mutable?
    pub current_color : PlayerColor,
    pub victory_state : VictoryState,
    pub age : i8, // how many balls were played?
    pub legal_moves : Vec<(i8, i8)>,
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
            legal_moves : self.legal_moves.clone(),
        }
    }
}

impl GameState {
    pub fn new() -> GameState {
        // The board is empty and all moves are legal
        let mut legal = Vec::new();
        for x in 0..4 {
            for y in 0..4 {
                legal.push((x, y));
            }
        }
        GameState {
            points : [PointState::Empty; 64],
            lines : [LineState::Empty; 76],
            current_color : PlayerColor::White,
            victory_state : VictoryState::Undecided,
            age : 0,
            legal_moves : legal,
        }
    }
}

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
        // Check if this really produced the right amount of points.
        // Assert the property, that point_box[i].flat_coordinate = i.
        assert_eq!(point_box.len(), 64);
        let mut i = 0;
        for p in &point_box {
            assert_eq!(p.flat_coordinate, i);
            i += 1;
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

        // Verify if the line_box has the right length.
        assert_eq!(line_box.len(), 76);


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

// fn expand(index:i8) -> (i8, i8, i8) {
//     return (index % 4, (index / 4) % 4, index / 16)
// }

pub fn execute_move(structure : &GameStructure, state : &mut GameState, play : Move) {
    match play {
        Move::Surrender   => state.victory_state = VictoryState::Win(!state.current_color),
        Move::Play {x, y} => play_at(structure, state, x, y),
    }
}

fn add_ball(line_state : LineState, new_color : PlayerColor) -> LineState {
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

fn z_value(game_state : &GameState, x : i8, y : i8) -> Option<i8> {
    for z in 0..4 {
        if game_state.points[flatten(x, y, z) as usize] == PointState::Empty {
            return Some(z)
        }
    }
    return None
}

fn play_at(structure : &GameStructure, state : &mut GameState, x:i8, y:i8) {
    let z = z_value(&state, x, y);
    let flat_coordinate = match z {
        Some(z) => flatten(x, y, z),
        None => panic!("Added a ball at ({}, {}), which is already full.", x, y)
    };
    // Place a colored piece at the coordinate
    state.points[flat_coordinate as usize] = PointState::Piece(state.current_color);
    // Update the legal moves, if the z-coordinate is 3
    // I make use of the fact that the z coordinate is occupying the top two bits.
    if flat_coordinate >= 4*4*3 {
        // As the legal moves will be mixed up during play, we need to search through all.
        for i in 0..state.legal_moves.len() {
            if state.legal_moves[i] == (x, y) {
                state.legal_moves.swap_remove(i);
                // Removes an element from anywhere in the vector and return it, replacing it with the last element.
                // This does not preserve ordering, but is O(1).
                break;
            }
        }
    }
    for line in structure.points[flat_coordinate as usize].lines.clone() {
        let line_state = add_ball(state.lines[line as usize], state.current_color);
        match line_state {
            LineState::Win(color) => state.victory_state = VictoryState::Win(color),
            _ => (),
        }
        state.lines[line as usize] = line_state;
    }
    state.age += 1;
    state.current_color = !state.current_color;
}
