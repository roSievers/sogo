
use game::{GameStructure};
use game;
use thread_synchronisation::{CoreEvent, UiEvent};
use constants::{LINES};//, PARALLELOGRAMS, PLUSSES};
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver};

use na::{Vector3, Point3, Point2};
use glfw;
use glfw::{Action, MouseButton, WindowEvent, Key};
use kiss3d::window::Window;
use kiss3d::light::Light;
use kiss3d::camera::{ArcBall, Camera};
use kiss3d::scene::SceneNode;

use game::{GameState, PlayerColor};

const BALL_DIAMMETER : f32 = 0.6;
const PLATE_HEIGHT : f32 = 0.4;
const ROD_LENGTH : f32 = 3.6; // In units of pieces.

enum UiState {
    WaitingForPlayerMove,
    WaitingForAiMove,
    GameOver,
}

/// Calculates the center of the piece in virtual 3D coordinates.
fn piece_position(x: i32, y: i32, z: i32) -> Vector3<f32> {
    return Vector3::new((x as f32)-1.5, (y as f32 + 0.5) * BALL_DIAMMETER *0.97, (z as f32)-1.5);
}

fn prepare_window() -> Window {
    let mut window = Window::new("Sogo");
    window.set_framerate_limit(Some(60));
    window.set_background_color(0.3, 0.3, 0.3);
    window.set_light(Light::StickToCamera);

    window
}

fn prepare_camera() -> ArcBall {
    let position        = Point3::new(6.0f32, 6.0, 6.0);
    let looking_towards = Point3::new(0.0f32, 1.5, 0.0);

    let mut camera = ArcBall::new(position, looking_towards);

    camera.rebind_drag_button(None);
    camera.rebind_rotate_button(Some(glfw::MouseButtonRight));
    camera.rebind_reset_key(None);

    camera
}

/// Creates the plate with 16 rods.
/// The scene's origin is located at the center of the plate's top
fn prepare_board() -> SceneNode {
    let mut node = SceneNode::new_empty();

    let mut plate     = node.add_cube(5.0, PLATE_HEIGHT, 5.0);
    plate.prepend_to_local_translation(&Vector3::new(0.0, -PLATE_HEIGHT/2.0, 0.0));
    plate.set_color(0.78, 0.65, 0.48);

    for i in 0..4 {
        for j in 0..4 {
            let height = BALL_DIAMMETER * ROD_LENGTH;
            let mut cylinder = node.add_cylinder(0.07f32, height);
            cylinder.prepend_to_local_translation(&Vector3::new((i as f32)-1.5, height/2.0, (j as f32)-1.5));
            cylinder.set_color(0.87, 0.72, 0.53);
        }
    }

    node
}

/// Creates a new 3D gamepiece and places it at the supplied position.
fn add_piece(scene : &mut SceneNode, x : i32, y : i32, z : i32, color : PlayerColor) -> SceneNode {
    let mut piece = scene.add_sphere(BALL_DIAMMETER/2.0);
    piece.prepend_to_local_translation(&piece_position(x, y, z));
    match color {
        PlayerColor::White => piece.set_color(1.0, 1.0, 1.0),
        PlayerColor::Black => piece.set_color(0.0, 0.0, 0.0),
    }

    piece
}

/// Creates a new 3D gamepiece and places it at the supplied position.
fn add_hint(scene : &mut SceneNode, x : i32, z : i32, color : PlayerColor) -> SceneNode {
    let mut piece = scene.add_sphere(BALL_DIAMMETER/2.0);
    piece.prepend_to_local_translation(&piece_position(x, 0, z));
    piece.prepend_to_local_translation(&Vector3::new(0.0, BALL_DIAMMETER * ROD_LENGTH, 0.0));
    match color {
        PlayerColor::White => piece.set_color(1.0, 1.0, 1.0),
        PlayerColor::Black => piece.set_color(0.0, 0.0, 0.0),
    }

    piece
}

fn placement_coordinate(window : &Window, camera : &ArcBall, cursor_position : (f64, f64)) -> Option<(i32, i32)> {
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


pub fn run_ui(core_sender : Sender<CoreEvent>, ui_receiver : Receiver<UiEvent>) {
    let structure = Rc::new(GameStructure::new(&LINES));
    let mut state = GameState::new(&structure);
    let mut history = ActionHistory::new();


    let mut window = prepare_window();
    let mut camera = prepare_camera();

    window.scene_mut().add_child(prepare_board());

    let mut current_placement_candidate = None;
    let mut placement_hint : Option<SceneNode> = None;

    while window.render_with_camera(&mut camera) {
        // Read the inter thread communication channel
        while let Ok(event) = ui_receiver.try_recv() {
            match event {
                UiEvent::RenderAction { action, color } => {
                    let (x, z) = action.unwrap();
                    let height = state.z_value(x, z).unwrap();
                    let new_piece = add_piece(window.scene_mut(), x as i32, height as i32, z as i32, color);
                    state.execute_action(&structure, &action);
                    history.add(action, new_piece);
                }
                remainder => println!("Unhandled thread event in UI: {:?}.", remainder),
            }
        }

        // Read the Kiss3D events.
        for event in window.events().iter() {
            match event.value {
                WindowEvent::CursorPos(x, y) => {
                    let new_placement_candidate = placement_coordinate(&window, &camera, (x, y));
                    if current_placement_candidate != new_placement_candidate {
                        // Destroy the current placement hint object.
                        if let Some(mut node) = placement_hint {
                            node.unlink(); // removes the node from the parent scene.
                            placement_hint = None
                        }
                        // Create new placement hint object, if applicable.
                        if let Some((x, y)) = new_placement_candidate {
                            if state.z_value(x as i8, y as i8).is_some() {
                                placement_hint = Some(add_hint(window.scene_mut(), x, y, state.current_color));
                            }
                        }
                        current_placement_candidate = new_placement_candidate;
                    }
                },
                WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                    if let Some((x_value, z_value)) = current_placement_candidate {
                        // Is placing a piece allowed?
                        if state.z_value(x_value as i8, z_value as i8).is_some() {
                            let action = game::Action::new(x_value as i8, z_value as i8);
                            core_sender.send(CoreEvent::Action{action : action, color : state.current_color}).unwrap();
                        }
                    }
                },
                WindowEvent::Key(Key::Left, _, Action::Release, _) => {
                    history.undo();
                    state = history.game_state(&structure);
                },
                WindowEvent::Key(Key::Right, _, Action::Release, _) => {
                    history.redo(window.scene_mut());
                    state = history.game_state(&structure);
                },
                _ => { }
            }
        }
    }
    core_sender.send(CoreEvent::Halt).unwrap();
}


struct ActionHistory {
    actions : Vec<(game::Action, SceneNode)>,
    // The current times is the amount of actions passed.
    // I.e. actions[current_time] is not executed and may not exist.
    // The current_time should never be larger than actions.len().
    current_time : usize,
}

impl ActionHistory {
    fn new() -> ActionHistory {
        ActionHistory {
            actions : Vec::new(),
            current_time : 0,
        }
    }
    /// Adds a new action to the ActionHistory.
    /// Stored actions after current_time are discarded.
    fn add(&mut self, action : game::Action, node : SceneNode) {
        while self.actions.len() > self.current_time {
            // The old nodes are already unlinked from their parents (not displayed).
            self.actions.pop();
        }
        self.actions.push((action, node));
        self.current_time += 1;
    }
    /// Creates a new GameState from scratch and applies all recorded actions up to current_time.
    /// This is in contrast to the 3D spheres which are individually created and removed.
    /// This function does not affect the graphical output at all.
    fn game_state(&self, structure : &GameStructure) -> GameState {
        let mut state = GameState::new(structure);
        for time in 0..self.actions.len() {
            if time >= self.current_time {
                break;
            }
            state.execute_action(structure, &self.actions[time].0);
        }
        state
    }
    fn undo(&mut self) {
        if self.current_time > 0 {
            self.current_time -= 1;
            self.actions[self.current_time].1.unlink();
        }
    }
    // The redo function need a SceneNode while the undo function doesn't because we need to know
    // the parent in order to relink the sphere. Unlink works without.
    fn redo(&mut self, scene : &mut SceneNode) {
        if self.current_time < self.actions.len() {
            // The Scene Node contains data: Rc<RefCell<SceneNodeData>>, so cloning it does just
            // return a new reference to the same data.
            scene.add_child(self.actions[self.current_time].1.clone());
            self.current_time += 1;
        }
    }
}
