/// This module holds the message structs used for thread communication.

use game;
use game::{Position2, VictoryState};

/// A core event is send to the core engine and processed by it.
#[derive(Clone, Debug)]
pub enum CoreEvent {
    #[allow(dead_code)]
    DebugOutput(String),
    Action {
        action: Position2,
        color: game::Color,
    },
    Halt,
}

#[derive(Clone, Debug)]
pub enum UiEvent {
    StartTurn,
    GameOver(VictoryState),
    RenderAction {
        action: Position2,
        color: game::Color,
    },
}
