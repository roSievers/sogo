/* This module refines a game::State object with a history.*/

use game;
use std::rc::Rc;

pub struct History {
    actions: Vec<game::Action>,
    playback_count: Option<usize>,
    pub state: game::State,
}

impl History {
    pub fn new(structure: Rc<game::Structure>) -> Self {
        History {
            actions: vec![],
            playback_count: None,
            state: game::State::new(structure),
        }
    }
    pub fn add(&mut self, action: game::Action) {
        self.actions.push(action);
        if self.playback_count.is_none() {
            self.state.execute(action);
        }
    }
    pub fn back(&mut self) -> Result<(), ()> {
        let current_count = self.playback_count.unwrap_or(self.actions.len());
        if current_count == 0 {
            return Err(());
        }
        let new_count = current_count - 1;
        self.playback_count = Some(new_count);

        self.state = game::State::new(self.state.structure.clone());
        for i in 0..new_count {
            self.state.execute(self.actions[i]);
        }
        Ok(())
    }
    pub fn forward(&mut self) -> Result<(), ()> {
        // You can only go forward if you are currently in the history replay mode.
        let current_count = self.playback_count.ok_or(())?;
        if current_count >= self.actions.len() {
            // You can't go forward if you are already at the latest position.
            return Err(());
        }
        let new_count = current_count + 1;
        self.playback_count = Some(new_count);
        self.state.execute(self.actions[current_count]);
        Ok(())
    }
    pub fn resume(&mut self) {
        if let Some(current_count) = self.playback_count {
            for i in current_count..self.actions.len() {
                self.state.execute(self.actions[i]);
            }
        } else {
            // We aren't in history playback mode so resuming doesn't do anything.
            return;
        }
    }
}
