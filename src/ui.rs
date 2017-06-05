
use game_view;
use game::GameStructure;
use game;
use thread_synchronisation::{CoreEvent, UiEvent};
use constants::LINES; //, PARALLELOGRAMS, PLUSSES};
use std::rc::Rc;

// Thread Communication
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};

use na::{Vector3, Point3, Point2, Translation3};
use glfw;
use glfw::{Action, MouseButton, WindowEvent, Key};
use kiss3d::window::Window;
use kiss3d::light::Light;
use kiss3d::camera::{ArcBall, Camera};
use kiss3d::scene::SceneNode;

enum UiState {
    WaitingForPlayerMove,
    WaitingForAiMove,
    GameOver,
}

fn prepare_window() -> Window {
    let mut window = Window::new("Sogo");
    window.set_framerate_limit(Some(60));
    window.set_background_color(0.3, 0.3, 0.3);
    window.set_light(Light::StickToCamera);

    window
}

fn prepare_camera() -> ArcBall {
    let position = Point3::new(6.0f32, 6.0, 6.0);
    let looking_towards = Point3::new(0.0f32, 1.5, 0.0);

    let mut camera = ArcBall::new(position, looking_towards);

    camera.rebind_drag_button(None);
    camera.rebind_rotate_button(Some(glfw::MouseButtonRight));
    camera.rebind_reset_key(None);

    camera
}


pub struct UiConnector {
    // A UiEvent is send TO the ui, a core event is send back.
    sender: Sender<UiEvent>,
    receiver: Receiver<CoreEvent>,
}


impl UiConnector {
    pub fn new() -> Self {
        let (my_sender, thread_receiver) = channel();
        let (thread_sender, my_receiver) = channel();

        thread::spawn(move || { run_ui(thread_sender, thread_receiver); });

        UiConnector {
            sender: my_sender,
            receiver: my_receiver,
        }
    }
    pub fn wait_for_action(&self) -> Result<game::Action, String> {
        self.sender.send(UiEvent::StartTurn).unwrap();

        // Blocks the thread until the user submits an action or quits.
        if let Ok(event) = self.receiver.recv() {
            match event {
                CoreEvent::DebugOutput(text) => {
                    println!("UI debug output: {}", text);
                    self.wait_for_action()
                }
                CoreEvent::Action { action, color } => Ok(action),
                CoreEvent::Halt => Err("Application window signaled 'Halt'.".to_owned()),
            }
        } else {
            Err("Application window closed.".to_owned())
        }
    }
    pub fn confirmed_action(&self,
                            action: game::Action,
                            color: game::PlayerColor)
                            -> Result<(), String> {
        self.sender
            .send(UiEvent::RenderAction {
                      action: action,
                      color: color,
                  })
            .unwrap();
        Ok(())
    }
}


pub fn run_ui(core_sender: Sender<CoreEvent>, ui_receiver: Receiver<UiEvent>) {
    let structure = Rc::new(GameStructure::new(&LINES));
    // let mut state = GameState::new(&structure);
    let mut game_state = game::State::new();
    //let mut history = ActionHistory::new();

    let mut view_state = game_view::State::empty();

    let mut window = prepare_window();
    let mut camera = prepare_camera();

    window.scene_mut().add_child(game_view::prepare_board());

    while window.render_with_camera(&mut camera) {
        // Read the inter thread communication channel
        while let Ok(event) = ui_receiver.try_recv() {
            match event {
                UiEvent::RenderAction { action, color } => {
                    let (x, z) = action.unwrap();
                    let height = game_state.column_height[(x + 4 * z) as usize];
                    let new_piece = game_view::add_piece(window.scene_mut(),
                                                         x as i32,
                                                         height as i32,
                                                         z as i32,
                                                         color);
                    game_state.execute(&structure, action);
                    // history.add(action, new_piece);
                }
                remainder => println!("Unhandled thread event in UI: {:?}.", remainder),
            }
        }

        // Read the Kiss3D events.
        for event in window.events().iter() {
            match event.value {
                WindowEvent::CursorPos(x, y) => {
                    let placement_candidate =
                        game_view::placement_coordinate(&window, &camera, (x, y));
                    view_state.placement_hint(window.scene_mut(),
                                              game_state.current_color,
                                              placement_candidate);
                }
                WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                    if let Some((x_value, z_value)) = view_state.placement_position() {
                        // Is placing a piece allowed?
                        if game_state.column_height[(x_value + 4 * z_value) as usize] <= 3 {
                            let action = game::Action::new(x_value as i8, z_value as i8);
                            core_sender
                                .send(CoreEvent::Action {
                                          action: action,
                                          color: game_state.current_color,
                                      })
                                .unwrap();
                        }
                    }
                }
                /*WindowEvent::Key(Key::Left, _, Action::Release, _) => {
                    history.undo();
                    state = history.game_state(&structure);
                }
                WindowEvent::Key(Key::Right, _, Action::Release, _) => {
                    history.redo(window.scene_mut());
                    state = history.game_state(&structure);
                }*/
                _ => {}
            }
        }
    }
    core_sender.send(CoreEvent::Halt).unwrap();
}

/*
struct ActionHistory {
    actions: Vec<(game::Action, SceneNode)>,
    // The current times is the amount of actions passed.
    // I.e. actions[current_time] is not executed and may not exist.
    // The current_time should never be larger than actions.len().
    current_time: usize,
}

impl ActionHistory {
    fn new() -> ActionHistory {
        ActionHistory {
            actions: Vec::new(),
            current_time: 0,
        }
    }
    /// Adds a new action to the ActionHistory.
    /// Stored actions after current_time are discarded.
    fn add(&mut self, action: game::Action, node: SceneNode) {
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
    fn game_state(&self, structure: &GameStructure) -> GameState {
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
    fn redo(&mut self, scene: &mut SceneNode) {
        if self.current_time < self.actions.len() {
            // The Scene Node contains data: Rc<RefCell<SceneNodeData>>, so cloning it does just
            // return a new reference to the same data.
            scene.add_child(self.actions[self.current_time].1.clone());
            self.current_time += 1;
        }
    }
}*/
