//use std::sync::Weak;
extern crate rand;
use rand::{thread_rng, Rng};


#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum PlayerColor {
    White,
    Black
}

fn flip_color(c : PlayerColor) -> PlayerColor {
    match c {
        PlayerColor::White => PlayerColor::Black,
        PlayerColor::Black => PlayerColor::White
    }
}

#[allow(dead_code)] // Dead code is allowed, because I want x, y, z even if I don't use it.
#[derive(Debug)]
struct Point {
    x : i8,
    y : i8,
    z : i8,
    flat_coordinate : i8,
    lines : Vec<i8> // This vector stores the IDs of all lines through it.
}

impl Point {
    // constructor, by convention
    pub fn new(x:i8, y:i8, z:i8) -> Point {
        Point {x:x, y:y, z:z, flat_coordinate:flatten(x, y, z), lines:Vec::new()} //, lines : Vec::new()}
    }
}

fn flatten(x:i8, y:i8, z:i8) -> i8 {
    return x + 4*y + 16*z
}

struct Line {
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
enum PointState {
    Piece(PlayerColor),
    Empty
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum LineState {
    Empty,
    Pure { color: PlayerColor, count: i8 },
    Mixed,
    Win(PlayerColor),
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum VictoryState {
    Undecided,
    Win(PlayerColor),
    Draw
}

struct GameState {
    points : [PointState; 64],
    lines  : [LineState; 76],  // something something mutable?
    current_color : PlayerColor,
    victory_state : VictoryState,
    age : i8, // how many balls were played?
    legal_moves : Vec<(i8, i8)>,
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

struct GameStructure {
    points : Vec<Point>,
    lines  : Vec<Line>
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
        None => panic!("Added a ball on a forbidden row")
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
    state.current_color = flip_color(state.current_color);
}

// fn play_at_random(structure : &GameStructure, state : &mut GameState) {
//     let moves = legal_moves(&state);
//     let position = thread_rng().choose(&moves);
//     match position {
//         Some(&(x, y)) => play_at(structure, state, x, y),
//         None => panic!("can't play on that board")
//     }
// }

enum Move {
    Play {x : i8, y : i8},
    Surrender
}

trait SogoAI {
    fn reset_game(&self);
    fn execute_move(&self, state : &GameState) -> Move;
}


#[allow(dead_code)] // Empty structs are unstable.
struct RandomSogoAI {
    alibi : i8,
}

impl RandomSogoAI {
    fn new() -> RandomSogoAI {
        RandomSogoAI { alibi : 42 }
    }
}

impl SogoAI for RandomSogoAI {
    fn reset_game(&self) { }
    fn execute_move(&self, state : &GameState) -> Move {
        let position = thread_rng().choose(&state.legal_moves);
        match position {
            Some(&(x, y)) => Move::Play {x:x, y:y},
            None => Move::Surrender
        }
    }
}

fn run_match<T : SogoAI, U : SogoAI>(structure : &GameStructure, white_player : &T, black_player : &U) -> GameState {
    let mut i = 0;

    let mut state = GameState::new();
    while state.victory_state == VictoryState::Undecided {
        if state.age == 64 {
            state.victory_state = VictoryState::Draw;
            return state;
        }
        let action = if i % 2 == 0 {white_player.execute_move(&state)} else {black_player.execute_move(&state)};
        match action {
            Move::Play { x, y} => play_at(structure, &mut state, x, y),
            Move::Surrender => state.victory_state = VictoryState::Win(flip_color(state.current_color))
        };
        i += 1;
    }
    // println!("{:?}", i);
    return state;
}

fn main() {
    let structure = GameStructure::new();
    let p1 = RandomSogoAI::new();
    let p2 = RandomSogoAI::new();
    for _ in 0..100000 {
        let state = run_match(&structure, &p1, &p2);
        //println!("The game took {:?} turns and ended with {:?}.", state.age, state.victory_state);
    }
    println!("done.")
}
