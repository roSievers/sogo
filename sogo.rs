//use std::sync::Weak;

#[derive(PartialEq, Eq, Debug)]
enum PlayerColor {
    White,
    Black
}

#[allow(dead_code)] // Dead code is allowed, because I want x, y, z even if I don't use it.
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
//
// enum PointState {
//     Piece(PlayerColor),
//     Empty
// }

#[derive(PartialEq, Eq, Debug)]
enum LineState {
    Empty,
    Pure { color: PlayerColor, count: i8 },
    Mixed,
    Win(PlayerColor),
}
//
// struct GameState {
//     points : [PointState; 64],
//     lines  : [LineState; 76]  // something something mutable?
// }

fn flatten(x:i8, y:i8, z:i8) -> i8 {
    return x + 4*y + 16*z
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

fn main() {
    // Initialize a vector of all Points.
    // This has the property, that point_box[i].flat_coordinate = i.
    let mut point_box = Vec::new();
    for z in 0..4 {
        for y in 0..4 {
            for x in 0..4 {
                point_box.push(Point::new(x, y, z));
            }
        }
    }
    // Check if this really produced the right amount of points.
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
    for line in line_box {
        for point in line.points.iter() {
            point_box[point.flat_coordinate as usize].lines.push(i);
        }
        i += 1;
    }

    // Do some testing with the LineState Code
    let mut test_state = LineState::Empty;
    test_state = add_ball(test_state, PlayerColor::White);
    println!("{:?}", test_state);
    test_state = add_ball(test_state, PlayerColor::White);
    println!("{:?}", test_state);
    test_state = add_ball(test_state, PlayerColor::White);
    println!("{:?}", test_state);
    test_state = add_ball(test_state, PlayerColor::Black);
    println!("{:?}", test_state);

    // for point in point_box {
    //     println!("{}", point.lines.len());
    // }
}
