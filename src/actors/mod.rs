//! Contains the implementations of the actors used for the game
//! like the player, coin, object, etc.

pub mod player;
pub mod types;
pub mod step_queue;
pub mod coin;
pub mod object;

use game_inputs::{InputEvent, Direction};