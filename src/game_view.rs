/* This module is supposed to isolate the state of the board. It should not know
anything about the camera position and scale. Changing the board state should
change the state of the 3D scene as well. To make this easier, the whole scene
is reconstructed whenever the scene changes. */
use na::{Vector3, Point2, Translation3};

use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use kiss3d::camera::{ArcBall, Camera};

use std::rc::Rc;

use game;
use game::{PlayerColor, Position2};
use replay::History;

const BALL_DIAMMETER: f32 = 0.6;
const PLATE_HEIGHT: f32 = 0.4;
const ROD_LENGTH: f32 = 3.6; // In units of pieces.

#[derive(PartialEq, Eq)]
pub enum Phase {
    Waiting,
    Input,
    GameOver(game::VictoryState),
}

pub struct State {
    pub replay: History,
    pub hint: Option<Position2>,
    pub phase: Phase,
}

impl State {
    pub fn empty(structure: Rc<game::Structure>) -> Self {
        State {
            replay: History::new(structure),
            hint: None,
            phase: Phase::Waiting,
        }

    }
}

/// Calculates the center of the piece in virtual 3D coordinates.
fn piece_position(x: i32, y: i32, z: i32) -> Vector3<f32> {
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
pub fn add_hint(scene: &mut SceneNode, position: Position2, color: PlayerColor) -> SceneNode {
    let (x, z) = position.coords();
    let mut piece = scene.add_sphere(BALL_DIAMMETER / 2.0);
    piece.append_translation(&Translation3::from_vector(piece_position(x as i32, 0, z as i32)));
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
                            -> Option<Position2> {
    let (x, y) = cursor_position;
    let (anchor, direction) = camera.unproject(&Point2::new(x as f32, y as f32), &window.size());
    // Fixme: might divide by zero.
    let lambda = ((BALL_DIAMMETER * 3.6f32) - anchor.y) / direction.y;
    let x_value = (anchor.x + lambda * direction.x + 1.5).round() as i32;
    let z_value = (anchor.z + lambda * direction.z + 1.5).round() as i32;

    if 0 <= x_value && x_value < 4 && 0 <= z_value && z_value < 4 {
        Some(Position2::new(x_value as u8, z_value as u8))
    } else {
        None
    }
}


pub fn render(target: &mut SceneNode, state: &State) {
    // First, clear the scene. This is a hack to make Kiss3D "stateless".
    let mut workaround_node_vec = vec![];
    target.apply_to_scene_nodes(&mut |node| workaround_node_vec.push(node.clone()));
    for mut node in workaround_node_vec {
        node.unlink();
    }
    // Render the empty board. (Baseplate & Sticks)
    target.add_child(prepare_board());

    // Render all pieces currently positioned.
    for (position, color) in state.replay.playback() {
        let mut piece = target.add_sphere(BALL_DIAMMETER / 2.0);
        let (x, y, z) = position.coords();
        piece.append_translation(&Translation3::from_vector(piece_position(x as i32,
                                                                           z as i32,
                                                                           y as i32)));
        match color {
            PlayerColor::White => piece.set_color(1.0, 1.0, 1.0),
            PlayerColor::Black => piece.set_color(0.0, 0.0, 0.0),
        }
    }

    // If there is a hint, render it.
    if let Some(position) = state.hint {
        add_hint(target, position, state.replay.state.current_color);
    }
}
