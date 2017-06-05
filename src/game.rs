use std::ops::Not;
use helpers::EqualityVerifier;

// The two dimensional position is a number between 0 and 15,
// the three dimensional position is a number between 0 and 63.
//
// But still, they should be differentiated and the type system must track this.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Position2(u8);
// Position3 is also known as FlatCoordinate in "legacy" code.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Position3(u8);

// Used for the GameStructure. This is a [bool; 64] in disguise.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Subset(u64);

impl Position2 {
    pub fn new(x: u8, y: u8) -> Self {
        debug_assert!(x <= 3 && y <= 3);
        Position2(x + 4 * y)
    }
    pub fn with_height(self, z: u8) -> Position3 {
        debug_assert!(z <= 3);
        Position3(self.0 + 16 * z)
    }
}

impl Position3 {
    pub fn new(x: u8, y: u8, z: u8) -> Self {
        debug_assert!(x <= 3 && y <= 3 && z <= 3);
        Position3(x + 4 * y + 16 * z)
    }
    pub fn column(self) -> Position2 {
        Position2(self.0 % 16)
    }
}

impl Subset {
    pub fn contains(self, position: Position3) -> bool {
        (self.0 >> position.0) % 2 == 1
    }
    pub fn iter(self) -> SubsetIterator {
        SubsetIterator {
            step_count: 0,
            shape: self.0,
        }
    }
}

pub struct SubsetIterator {
    step_count: u8,
    shape: u64,
}

impl Iterator for SubsetIterator {
    type Item = Position3;
    fn next(&mut self) -> Option<Self::Item> {
        if self.step_count == 64 {
            None
        } else {
            if self.step_count != 0 {
                self.shape /= 2;
            }

            self.step_count += 1;

            if self.shape % 2 == 1 {
                Some(Position3(self.step_count - 1))
            } else {
                self.next()
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum PlayerColor {
    White,
    Black,
}

// This implementation allows PlayerColor::White == !PlayerColor::Black.
impl Not for PlayerColor {
    type Output = PlayerColor;

    fn not(self) -> PlayerColor {
        match self {
            PlayerColor::White => PlayerColor::Black,
            PlayerColor::Black => PlayerColor::White,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Point {
    pub flat_coordinate: usize,
    pub lines: Vec<usize>, // This vector stores the IDs of all lines through it.
}

impl Point {
    // constructor, by convention
    pub fn new(x: i8, y: i8, z: i8) -> Point {
        Point {
            flat_coordinate: flatten(x, y, z),
            lines: Vec::new(),
        } //, lines : Vec::new()}
    }

    // Allow the coordinate getters to stay around for debugging purposes.
    #[allow(dead_code)]
    pub fn get_x(&self) -> usize {
        self.flat_coordinate % 4
    }
    #[allow(dead_code)]
    pub fn get_y(&self) -> usize {
        (self.flat_coordinate / 4) % 4
    }
    #[allow(dead_code)]
    pub fn get_z(&self) -> usize {
        self.flat_coordinate / 16
    }
}

pub fn flatten(x: i8, y: i8, z: i8) -> usize {
    return (x + 4 * y + 16 * z) as usize;
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Action {
    // FIXME: This should store a Position2 instead.
    Play { x: i8, y: i8 },
    Surrender,
}

impl Action {
    pub fn new(x: i8, y: i8) -> Action {
        Action::Play { x: x, y: y }
    }
    // FIXME: This should take a Position2 instead
    pub fn flat(flat_coordinate: i8) -> Action {
        Action::new(flat_coordinate % 4, flat_coordinate / 4)
    }
    pub fn unwrap(&self) -> (i8, i8) {
        match self {
            &Action::Play { x: x, y: y } => (x, y),
            _ => panic!("Unwrapping the Action failed."),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum PointState {
    Piece(PlayerColor),
    Empty,
}

// TODO: Move this into AI. There is no reason to store it inside the game::State.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum LineState {
    Empty,
    Pure { color: PlayerColor, count: i8 },
    Mixed,
    Win(PlayerColor),
}

impl LineState {
    fn add_ball_functional(line_state: LineState,
                           new_color: PlayerColor,
                           max_size: i8)
                           -> LineState {
        match line_state {
            LineState::Empty => {
                LineState::Pure {
                    color: new_color,
                    count: 1,
                }
            }
            LineState::Pure {
                color: current_color,
                count: old_count,
            } => {
                if current_color != new_color {
                    LineState::Mixed
                } else {
                    if old_count == max_size - 1 {
                        LineState::Win(current_color)
                    } else {
                        LineState::Pure {
                            color: current_color,
                            count: old_count + 1,
                        }
                    }
                }
            }
            LineState::Mixed => LineState::Mixed,
            LineState::Win(_) => panic!("A filled line can't accept any more balls."),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum VictoryState {
    Undecided,
    Win(PlayerColor),
    Draw,
}

impl VictoryState {
    pub fn as_float(&self, perspective: PlayerColor) -> f32 {
        match *self {
            VictoryState::Undecided => 0.5,
            VictoryState::Draw => 0.5,
            VictoryState::Win(color) => if color == perspective { 1.0 } else { 0.0 },
        }
    }
    pub fn active(&self) -> bool {
        match *self {
            VictoryState::Undecided => true,
            _ => false,
        }
    }
}

// TODO: Move this elsewhere
#[derive(Debug, Copy, Clone)]
pub struct VictoryStats {
    pub white: i32,
    pub black: i32,
    pub draws: i32,
}

impl VictoryStats {
    pub fn new() -> VictoryStats {
        VictoryStats {
            white: 0,
            black: 0,
            draws: 0,
        }
    }
}

#[derive(Clone)]
pub struct GameStructure {
    pub points: Vec<Point>,
    pub source: Vec<Subset>,
    victory_object_count: usize,
    // All victory objects need to be of the same size. This is an implementation restriction.
    victory_object_size: i8,
}

impl GameStructure {
    pub fn new(victory_objects: &[u64]) -> GameStructure {
        // Initialize a vector of all Points.
        let mut point_box = Vec::new();
        for z in 0..4 {
            for y in 0..4 {
                for x in 0..4 {
                    point_box.push(Point::new(x, y, z));
                }
            }
        }

        // Make sure the victory object count is the same for each object.
        let mut object_count = EqualityVerifier::new();

        // Refenence the line ID in the points.
        for line_id in 0..victory_objects.len() {
            // rep is a u64 encoding of the line.
            let mut rep = victory_objects[line_id];
            object_count.update(rep.count_ones() as i8);
            let mut flat = 0;
            while rep > 0 {
                // A one indicates that this line has a point at that particular position.
                if rep % 2 == 1 {
                    point_box[flat].lines.push(line_id);
                }
                rep /= 2;
                flat += 1;
            }
        }

        let source = victory_objects.iter().map(|v| Subset(*v)).collect();

        GameStructure {
            points: point_box,
            source: source,
            victory_object_count: victory_objects.len(),
            victory_object_size: object_count.unwrap(),
        }
    }
}

// The new State, replaces the old GameState
pub struct State {
    pub points: [PointState; 64],
    pub current_color: PlayerColor,
    pub age: i8, // How many actions were played?
    // Everything below here is cached information.
    pub victory_state: VictoryState,
    // Caches the column height (0, 1, 2, 3) to quickly determine available moves.
    pub column_height: [u8; 16],
}

impl State {
    pub fn new() -> Self {
        State {
            points: [PointState::Empty; 64],
            current_color: PlayerColor::White,
            age: 0,
            victory_state: VictoryState::Undecided,
            column_height: [0; 16],
        }
    }
    fn at(&self, position: Position3) -> PointState {
        self.points[position.0 as usize]
    }
    pub fn execute(&mut self, structure: &GameStructure, action: Action) {
        match action {
            Action::Surrender => self.victory_state = VictoryState::Win(!self.current_color),
            Action::Play { x, y } => self.insert(structure, Position2::new(x as u8, y as u8)),
        }
    }
    // Panics, if the column is already full.
    pub fn insert(&mut self, structure: &GameStructure, column: Position2) {
        let position = {
            let z = self.column_height.get_mut(column.0 as usize).unwrap();
            let position = column.with_height(*z);
            *z += 1;
            position
        };
        self.points[position.0 as usize] = PointState::Piece(self.current_color);
        self.age += 1;
        self.current_color = !self.current_color;
        self.update_victory_state(structure, position);
    }
    fn update_victory_state(&mut self, structure: &GameStructure, position: Position3) {
        // TODO: Will be faster, if I properly use the reverse lookup table.
        for subset in &structure.source {
            if subset.contains(position) {
                if subset
                       .iter()
                       .all(|pos2| self.at(pos2) == PointState::Piece(self.current_color)) {
                    self.victory_state = VictoryState::Win(self.current_color);
                    return;
                }
            }
        }
    }
    // TODO: Convert this into an iterator when `impl Trait` lands.
    // This is predicted for 1.20 on 31st of August 2017.
    // https://internals.rust-lang.org/t/rust-release-milestone-predictions/4591
    // Or change to nightly at any point :P
    // This should actually return `Iterator<Action>` and avoid allocation :-I
    pub fn legal_actions(&self) -> Vec<Action> {
        // Fuck missing impl trait xP
        self.column_height
            .iter()
            .enumerate()
            .filter(|&(_, h)| *h <= 3)
            .map(|(i, _)| Action::flat(i as i8))
            .collect()
    }
}

// Once [T; 64] becomes Clone, not just Copy, this can be derived.
impl Clone for State {
    fn clone(&self) -> Self {
        State {
            points: self.points,
            current_color: self.current_color,
            age: self.age,
            victory_state: self.victory_state,
            column_height: self.column_height,
        }
    }
}
