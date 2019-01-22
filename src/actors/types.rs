//! Contains the Actor types within the game, as well as the collision data
//! structure required for collision handling.
use std::cell::Cell;
use ggez::nalgebra as na;

#[derive(Clone, Copy)]
pub enum ActorType {
	Player,
	Coin,
    Object,
}

#[derive(Clone, Debug)]
pub struct CollisionObjectData {
    pub name: &'static str,
    pub velocity: Option<Cell<na::Vector2<f32>>>,
}

impl CollisionObjectData {
    pub fn new(name: &'static str, velocity: Option<na::Vector2<f32>>) -> CollisionObjectData {
        let init_velocity;
        if let Some(velocity) = velocity {
            init_velocity = Some(Cell::new(velocity))
        } else {
            init_velocity = None
        }

        CollisionObjectData {
            name: name,
            velocity: init_velocity,
        }
    }
}