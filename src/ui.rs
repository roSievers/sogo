
use game_view;
use game::Position2;
use game;
use thread_synchronisation::{CoreEvent, UiEvent};
use constants::LINES; //, PARALLELOGRAMS, PLUSSES};
use std::rc::Rc;
use replay;

// Thread Communication
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};

use na::Point3;
use glfw;
use glfw::{Action, MouseButton, WindowEvent, Key};
// TODO: Find a better, stateless 3D option than this.
// Being able to place text would be amazing as well.
use kiss3d::window::Window;
use kiss3d::light::Light;
use kiss3d::camera::ArcBall;

#[derive(PartialEq, Eq)]
enum UiState {
    Waiting,
    Input,
    GameOver(game::VictoryState),
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
                CoreEvent::Action { action, .. } => Ok(action),
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
    pub fn game_over(&self, victory_state: game::VictoryState) {
        self.sender.send(UiEvent::GameOver(victory_state)).unwrap();
    }
    pub fn wait_for_halt(&self) {
        // Blocks the thread until the user submits an action or quits.
        if let Ok(event) = self.receiver.recv() {
            match event {
                CoreEvent::Halt => (),
                remainder => {
                    println!("UI returned an event after the game finished: {:?}",
                             remainder);
                    self.wait_for_halt()
                }
            }
        } else {
            ()
        }

    }
}


pub fn run_ui(core_sender: Sender<CoreEvent>, ui_receiver: Receiver<UiEvent>) {
    let structure = Rc::new(game::Structure::new(&LINES));
    let mut replay = replay::History::new();

    let mut view_state = game_view::State::empty();

    let mut window = prepare_window();
    let mut camera = prepare_camera();

    let mut ui_state = UiState::Waiting;

    window.scene_mut().add_child(game_view::prepare_board());

    while window.render_with_camera(&mut camera) {
        // Read the inter thread communication channel
        while let Ok(event) = ui_receiver.try_recv() {
            match event {
                UiEvent::RenderAction { action, color } => {
                    let (x, z) = action.unwrap().coords();
                    let height = replay.state.column_height[(x + 4 * z) as usize];
                    game_view::add_piece(window.scene_mut(),
                                         x as i32,
                                         height as i32,
                                         z as i32,
                                         color);
                    replay.add(&structure, action);
                    // history.add(action, new_piece);
                }
                UiEvent::StartTurn => {
                    ui_state = UiState::Input;
                }
                UiEvent::GameOver(victory_state) => {
                    ui_state = UiState::GameOver(victory_state);
                }
            }
        }

        // Read the Kiss3D events.
        for event in window.events().iter() {
            match event.value {
                WindowEvent::CursorPos(x, y) => {
                    if ui_state == UiState::Input {
                        let placement_candidate =
                            game_view::placement_coordinate(&window, &camera, (x, y));
                        view_state.placement_hint(window.scene_mut(),
                                                  replay.state.current_color,
                                                  placement_candidate);
                    }
                }
                WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                    if ui_state == UiState::Input {
                        if let Some((x_value, z_value)) = view_state.placement_position() {
                            // Is placing a piece allowed?
                            if replay.state.column_height[(x_value + 4 * z_value) as usize] <= 3 {
                                let action = Position2::new(x_value as u8, z_value as u8).into();
                                core_sender
                                    .send(CoreEvent::Action {
                                              action: action,
                                              color: replay.state.current_color,
                                          })
                                    .unwrap();
                            }
                            ui_state = UiState::Waiting;
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
