//! `game_input` contains the input handler for the player.
use ggez::event::{self, Keycode};

/// `Direction` containst he available directions for the player to move.
/// Currently limited to Left and Right, but can be expanded upon.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum Direction {
    Left,
    Right,
}

/// The functions of `Direction` allow for applying a direction to key
/// inputs via the `Keycode`, and provide a +/- multiplier to allow for
/// direction along the X axis.
impl Direction {
    pub fn movement(self) -> f64 {
        match self {
            Direction::Left => -1.0,
            Direction::Right => 1.0,
        }
    }

    fn fromKey(keycode: Keycode) -> Option<Direction> {
        match keycode {
            Keycode::Left => Some(Direction::Left),
            Keycode::Right => Some(Direction::Right),
            _ => None,
        }
    }
}

/// `InputEvent` is utlized to generate events for various movement types.
pub enum InputEvent {
    UpdateMovement(Option<Direction>),
    PressJump,
    TimeUpdate,
    Landed,
}

/// `GameInput` contains a vector of key holds to help ensure constant movement
/// of the player character while the keys are being pressed.
pub struct GameInput {
    held_dirs: Vec<Direction>
}

impl GameInput {
    pub fn new() -> GameInput { GameInput {held_dirs: Vec::new() }}

    pub fn key_down_event(&mut self, keycode: Keycode) -> Option<InputEvent> {
        if let Some(direction) = Direction::fromKey(keycode) {
            self.held_dirs.push(direction);
            Some(InputEvent::UpdateMovement(Some(direction)))
        }
        else if keycode == Keycode::Space {
            Some(InputEvent::UpdateMovement(self.held_dirs()))

        }
        else {
            None
        }
    }

    pub fn key_up_event (&mut self, keycode: Keycode) -> Option<InputEvent> {
        if let Some(direction) = Direction::fromKey(keycode) {
            self.held_dirs.retain(|&d| d != direction);
            Some(InputEvent::UpdateMovement(self.held_dirs()))
        }
        else{
            None
        }
    }

    pub fn held_dirs(&self) -> Option<Direction> {
        self.held_dirs.last().cloned()
    }
}