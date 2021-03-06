use std::ops::{AddAssign, Not};
use std::sync::Arc;

// The two dimensional position is a number between 0 and 15,
// the three dimensional position is a number between 0 and 63.
//
// But still, they should be differentiated and the type system must track this.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Position2(pub u8);
// Position3 is also known as FlatCoordinate in "legacy" code.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Position3(pub u8);

// Used for the Structure. This is a [bool; 64] in disguise.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Subset(pub u64);

impl Position2 {
    pub fn new(x: u8, y: u8) -> Self {
        debug_assert!(x <= 3 && y <= 3);
        Position2(x + 4 * y)
    }
    pub fn with_height(self, z: u8) -> Position3 {
        debug_assert!(z <= 3);
        Position3(self.0 + 16 * z)
    }
    pub fn coords(self) -> (u8, u8) {
        (self.0 % 4, self.0 / 4)
    }
}

impl Position3 {
    #[allow(dead_code)]
    pub fn new(x: u8, y: u8, z: u8) -> Self {
        debug_assert!(x <= 3 && y <= 3 && z <= 3);
        Position3(x + 4 * y + 16 * z)
    }
    pub fn coords(self) -> (u8, u8, u8) {
        (self.0 % 4, (self.0 / 4) % 4, self.0 / 16)
    }
}

impl From<Position3> for Position2 {
    fn from(position3: Position3) -> Position2 {
        Position2(position3.0 % 16)
    }
}

impl Subset {
    #[allow(dead_code)]
    // This is used by the tests and in general a nice function to have
    // around.
    pub fn contains(self, position: Position3) -> bool {
        (self.0 >> position.0) % 2 == 1
    }
    pub fn iter(self) -> SubsetIterator {
        SubsetIterator {
            step_count: 0,
            shape: self.0,
        }
    }
    pub fn win_state(self, state: &State) -> LineState {
        let mut stats = SubsetStats {
            color: None,
            objects: 0,
            full: true,
            mixed: false,
        };
        for point in self.iter().map(|p| state.at(p)) {
            stats += point;
        }
        if stats.mixed {
            LineState::Mixed
        } else if stats.full {
            LineState::Win(stats.color.unwrap())
        } else if stats.color == None {
            LineState::Empty
        } else {
            LineState::Pure {
                color: stats.color.unwrap(),
                count: stats.objects as i8,
            }
        }
    }
}

#[derive(Debug)]
struct SubsetStats {
    color: Option<Color>,
    objects: u8,
    full: bool,
    mixed: bool,
}

impl AddAssign<PointState> for SubsetStats {
    fn add_assign(&mut self, new_point: PointState) {
        match new_point {
            PointState::Empty => self.full = false,
            PointState::Piece(color) => {
                self.objects += 1;
                match self.color {
                    None => self.color = Some(color),
                    Some(new_color) => {
                        if color != new_color {
                            self.mixed = true
                        }
                    }
                }
            }
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
pub enum Color {
    White,
    Black,
}

// This implementation allows Color::White == !Color::Black.
impl Not for Color {
    type Output = Color;

    fn not(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum PointState {
    Piece(Color),
    Empty,
}

// TODO: Move this into AI. There is no reason to store it inside the game::State.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum LineState {
    Empty,
    Pure { color: Color, count: i8 },
    Mixed,
    Win(Color),
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum VictoryState {
    Undecided,
    Win {
        winner: Color,
        reason: Option<Subset>,
    },
    Draw,
}

impl VictoryState {
    pub fn active(&self) -> bool {
        match *self {
            VictoryState::Undecided => true,
            _ => false,
        }
    }
    pub fn scoring(&self, color: Color) -> Option<i8> {
        match *self {
            VictoryState::Win { winner, .. } => if winner == color { Some(1) } else { Some(-1) },
            VictoryState::Draw => Some(0),
            VictoryState::Undecided => None,
        }
    }
}

// TODO: Move this elsewhere, helpers for a start. Implement some traits
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


pub struct Structure {
    // A vector of all Subsets, complete one to win the game.
    pub source: Vec<Subset>,
    // Contains a lookup table, given a Position3, this returns a vector of indices.
    // The indices tell you which Subsets contain the Position3.
    pub reverse: [Vec<usize>; 64],
    // The size of a victory object. While not technically necessary, having
    // uniform victory objects seems like a reasonable restriction.
    pub object_size: u8,
}

impl Structure {
    pub fn new(victory_objects: &[u64]) -> Structure {
        use helpers::EqualityVerifier;

        // Convert raw u64 into Subset objects. (Which are u64 with extra structure.)
        let source: Vec<Subset> = victory_objects.iter().map(|v| Subset(*v)).collect();
        // Unfortunately, [vec![]; 64] does not work :-/
        let mut reverse = [
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ];

        let mut object_size = EqualityVerifier::NoValue;

        for (index, subset) in source.iter().enumerate() {
            let mut subset_size = 0;
            for position in subset.iter() {
                reverse[position.0 as usize].push(index);
                subset_size += 1;
            }
            object_size = object_size.update(subset_size);
        }

        Structure {
            source,
            reverse,
            object_size: object_size.unwrap(),
        }
    }
}

pub struct State {
    pub points: [PointState; 64],
    pub current_color: Color,
    pub age: u8, // How many actions were played?
    // Everything below here is cached information.
    pub victory_state: VictoryState,
    // Caches the column height (0, 1, 2, 3) to quickly determine available moves.
    pub column_height: [u8; 16],
    pub structure: Arc<Structure>,
}

impl State {
    pub fn new(structure: Arc<Structure>) -> Self {
        State {
            points: [PointState::Empty; 64],
            current_color: Color::White,
            age: 0,
            victory_state: VictoryState::Undecided,
            column_height: [0; 16],
            structure,
        }
    }
    pub fn at(&self, position: Position3) -> PointState {
        self.points[position.0 as usize]
    }
    pub fn execute(&mut self, column: Position2) {
        let position = self.insert(column);
        let color = !self.current_color;
        self.update_victory_state(position, color);
    }
    // Panics, if the column is already full.
    // This does NOT update the victory state. Use `.execute` for this.
    pub fn insert(&mut self, column: Position2) -> Position3 {
        let position = {
            let z = self.column_height.get_mut(column.0 as usize).unwrap();
            let position = column.with_height(*z);
            *z += 1;
            position
        };
        self.points[position.0 as usize] = PointState::Piece(self.current_color);
        self.age += 1;
        self.current_color = !self.current_color;
        position
    }
    fn update_victory_state(&mut self, position: Position3, color: Color) {
        for subset_index in self.structure.reverse[position.0 as usize].iter() {
            let subset = self.structure.source[*subset_index];
            if subset.iter().all(|pos2| {
                self.at(pos2) == PointState::Piece(color)
            })
            {
                self.victory_state = VictoryState::Win {
                    winner: color,
                    reason: Some(subset),
                };
                return;
            }
        }
        if self.age == 64 {
            self.victory_state = VictoryState::Draw;
        }
    }
    // TODO: Remove the Box when `impl Trait` lands.
    // This is predicted for 1.20 on 31st of August 2017.
    // https://internals.rust-lang.org/t/rust-release-milestone-predictions/4591
    // Or change to nightly at any point :P
    pub fn legal_actions<'a>(&'a self) -> Box<Iterator<Item = Position2> + 'a> {
        // Fuck missing impl trait xP
        Box::new(
            self.column_height
                .iter()
                .enumerate()
                .filter(|&(_, h)| *h <= 3)
                .map(|(i, _)| Position2(i as u8)),
        )
    }
    pub fn column_full(&self, column: Position2) -> bool {
        self.column_height[column.0 as usize] == 4
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
            structure: self.structure.clone(),
        }
    }
}
