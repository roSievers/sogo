/// This module holds the message structs used for thread communication.

use game::{Action, PlayerColor, VictoryState};

/// A core event is send to the core engine and processed by it.
#[derive(Clone, Debug)]
pub enum CoreEvent {
    #[allow(dead_code)]
    DebugOutput(String),
    Action { action : Action, color : PlayerColor },
    Halt,
}

#[derive(Clone, Debug)]
pub enum UiEvent {
    StartTurn,
    GameOver(VictoryState),
    RenderAction { action : Action, color : PlayerColor },
}
