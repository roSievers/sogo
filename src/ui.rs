
use game_view;
use game_view::Phase;
use game;
use thread_synchronisation::{CoreEvent, UiEvent};
use constants::LINES; //, PARALLELOGRAMS, PLUSSES};
use std::rc::Rc;

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
    pub fn confirmed_action(&self, action: game::Action, color: game::Color) -> Result<(), String> {
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
                    println!(
                        "UI returned an event after the game finished: {:?}",
                        remainder
                    );
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

    let mut view_state = game_view::State::empty(structure.clone());

    let mut window = prepare_window();
    let mut camera = prepare_camera();

    while window.render_with_camera(&mut camera) {
        // Read the inter thread communication channel
        while let Ok(event) = ui_receiver.try_recv() {
            match event {
                UiEvent::RenderAction { action, .. } => {
                    view_state.replay.add(action);
                }
                UiEvent::StartTurn => {
                    view_state.phase = Phase::Input;
                }
                UiEvent::GameOver(victory_state) => {
                    view_state.phase = Phase::GameOver(victory_state);
                }
            }
        }

        // Read the Kiss3D events.
        for event in window.events().iter() {
            match event.value {
                WindowEvent::CursorPos(x, y) => {
                    if view_state.phase == Phase::Input && view_state.replay.is_resumed() {
                        let placement_candidate =
                            game_view::placement_coordinate(&window, &camera, (x, y));
                        view_state.hint = placement_candidate;
                    }
                }
                WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                    if let Some(position) = view_state.hint {
                        assert_eq!(view_state.phase, Phase::Input);
                        view_state.hint = None;
                        // Is placing a piece allowed?
                        if view_state.replay.state.column_height[position.0 as usize] <= 3 {
                            let action = position.into();
                            core_sender
                                .send(CoreEvent::Action {
                                    action: action,
                                    color: view_state.replay.state.current_color,
                                })
                                .unwrap();
                        }
                        view_state.phase = Phase::Waiting;

                    }
                }
                WindowEvent::Key(Key::Left, _, Action::Release, _) => {
                    let result = view_state.replay.back();
                    view_state.hint = None;
                    if result.is_err() {
                        // TODO: play an error sound.
                    }
                }
                WindowEvent::Key(Key::Right, _, Action::Release, _) => {
                    let result = view_state.replay.forward();
                    view_state.hint = None;
                    if result.is_err() {
                        // TODO: play an error sound.
                    }
                }
                WindowEvent::Key(Key::Space, _, Action::Release, _) => {
                    view_state.replay.resume();
                }
                _ => {}
            }
        }

        game_view::render(window.scene_mut(), &view_state);
    }
    core_sender.send(CoreEvent::Halt).unwrap();
}
