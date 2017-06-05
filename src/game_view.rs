
use na::{Vector3, Point3, Point2, Translation3};

use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use kiss3d::light::Light;
use kiss3d::camera::{ArcBall, Camera};

use std::mem;

use game::PlayerColor;

const BALL_DIAMMETER: f32 = 0.6;
const PLATE_HEIGHT: f32 = 0.4;
const ROD_LENGTH: f32 = 3.6; // In units of pieces.

struct Hint {
    x: i32,
    y: i32,
    node: SceneNode,
}

pub struct State {
    hint: Option<Hint>,
}

impl State {
    pub fn empty() -> Self {
        State { hint: None }
    }
    pub fn placement_hint(&mut self, scene: &mut SceneNode, color: PlayerColor, position: Option<(i32, i32)>) {
        let current_position = self.hint.as_ref().map(|hint| (hint.x, hint.y));

        if current_position == position {
            return
        }

        // Remove the node from the parent scene, if there is a node.
        self.hint.as_mut().map(|hint| hint.node.unlink());

        // Create a new hint object.
        self.hint = position.map(|(x, y)| {
            // TODO: Some verification
            // if state.z_value(x as i8, y as i8).is_some() {
            let node = add_hint(scene, x, y, color);
            Hint {x, y, node}
        })
    }
    // This dropes the current placement hint and returns its coordinates.
    pub fn placement_position(&mut self) -> Option<(i32, i32)> {
        let mut old_hint = None;

        mem::swap(&mut old_hint, &mut self.hint);

        old_hint.as_mut().map(|hint| {
            hint.node.unlink();
            (hint.x, hint.y)
        })
    }
}

/// Calculates the center of the piece in virtual 3D coordinates.
pub fn piece_position(x: i32, y: i32, z: i32) -> Vector3<f32> {
    return Vector3::new((x as f32) - 1.5,
                        (y as f32 + 0.5) * BALL_DIAMMETER * 0.97,
                        (z as f32) - 1.5);
}


/// Creates the plate with 16 rods.
/// The scene's origin is located at the center of the plate's top
pub fn prepare_board() -> SceneNode {
    let mut node = SceneNode::new_empty();

    let mut plate = node.add_cube(5.0, PLATE_HEIGHT, 5.0);
    plate.append_translation(&Translation3::new(0.0, -PLATE_HEIGHT / 2.0, 0.0));
    plate.set_color(0.78, 0.65, 0.48);

    for i in 0..4 {
        for j in 0..4 {
            let height = BALL_DIAMMETER * ROD_LENGTH;
            let mut cylinder = node.add_cylinder(0.07f32, height);
            cylinder.append_translation(&Translation3::new((i as f32) - 1.5,
                                                           height / 2.0,
                                                           (j as f32) - 1.5));
            cylinder.set_color(0.87, 0.72, 0.53);
        }
    }

    node
}

/// Creates a new 3D gamepiece and places it at the supplied position.
pub fn add_piece(scene: &mut SceneNode, x: i32, y: i32, z: i32, color: PlayerColor) -> SceneNode {
    let mut piece = scene.add_sphere(BALL_DIAMMETER / 2.0);
    piece.append_translation(&Translation3::from_vector(piece_position(x, y, z)));
    match color {
        PlayerColor::White => piece.set_color(1.0, 1.0, 1.0),
        PlayerColor::Black => piece.set_color(0.0, 0.0, 0.0),
    }

    piece
}

/// Creates a new 3D gamepiece and places it at the supplied position.
pub fn add_hint(scene: &mut SceneNode, x: i32, z: i32, color: PlayerColor) -> SceneNode {
    let mut piece = scene.add_sphere(BALL_DIAMMETER / 2.0);
    piece.append_translation(&Translation3::from_vector(piece_position(x, 0, z)));
    piece.append_translation(&Translation3::new(0.0, BALL_DIAMMETER * ROD_LENGTH, 0.0));
    match color {
        PlayerColor::White => piece.set_color(1.0, 1.0, 1.0),
        PlayerColor::Black => piece.set_color(0.0, 0.0, 0.0),
    }

    piece
}

pub fn placement_coordinate(window: &Window,
                            camera: &ArcBall,
                            cursor_position: (f64, f64))
                            -> Option<(i32, i32)> {
    let (x, y) = cursor_position;
    let (anchor, direction) = camera.unproject(&Point2::new(x as f32, y as f32), &window.size());
    // Fixme: might divide by zero.
    let lambda = ((BALL_DIAMMETER * 3.6f32) - anchor.y) / direction.y;
    let x_value = (anchor.x + lambda * direction.x + 1.5).round() as i32;
    let z_value = (anchor.z + lambda * direction.z + 1.5).round() as i32;

    if 0 <= x_value && x_value < 4 && 0 <= z_value && z_value < 4 {
        Some((x_value, z_value))
    } else {
        None
    }
}
