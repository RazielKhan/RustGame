//! `Coin` is the object that contains all of the required detail for the coin
//! pick up object in the game.

use std::env;
use std::path;
use std::collections::LinkedList;
use std::time::{Duration, Instant};

use ggez::conf;
use ggez::event;
use ggez::graphics;
use ggez::graphics::{Rect, Vector2, Point2};
use ggez::{Context, GameResult};
use ggez::nalgebra as na;
use ggez::nalgebra::{Isometry2};

use actors::types::ActorType;
use game_inputs::{Direction, InputEvent};
use ncollide::world::{CollisionObjectHandle, CollisionWorld2};

/// Borrowed from GGEZ Astroblasto example
/// This is used to transalte the world coordinate system which has both Y == 0
/// and X == 0 being the origin (center of the screen), and converts it to the
/// screen coordinate system which has the origin in the upper left of the
/// screen with Y inverted (increasing in a downward direction).
/// This helps with converting all items being rendred from the top-left.
fn world_to_screen_coords(screen_width: u32, screen_height: u32, point: Vector2) -> Vector2 {
    let width = screen_width as f32;
    let height = screen_height as f32;
    let x = point.x + width / 2.0;
    let y = height - (point.y + height / 2.0);
    Vector2::new(x, y)
}

#[derive(Clone, Copy)]
pub struct Coin {
	pub tag: ActorType,
	pub pos: Vector2,
	pickedup: bool,
	debug: bool,
	col_handle: Option<CollisionObjectHandle>,
}

/// `Coin` is an interactable object that can give points. Currently, points are
/// handled in the main collision handle function; however, the points need to
/// be implemented as a `Coin` attribute to reduce redundancy and simplify code.
impl Coin {
	pub fn new(pos: Vector2) -> Coin {
		let mut coin = Coin {
			tag: ActorType::Coin,
			pos: Vector2::new(1920./2. - 750., -1080./4.),
			pickedup: false,
			debug: true,
			col_handle: None,
		};
		
		(coin)
	}

	// `update()` ensures the collision handle stays in the same location as the rendered coin object.
	pub fn update(&mut self, ctx: &mut Context, world: &mut CollisionWorld2<f32, ()>) {
		let position = world_to_screen_coords(ctx.conf.window_mode.width, ctx.conf.window_mode.height, self.pos);
		world.set_position(self.col_handle.unwrap(), Isometry2::new(Vector2::new(position.x.clone(), position.y.clone()), 0.));
	}

	pub fn set_col_handle(&mut self, col_handle: CollisionObjectHandle) {
		self.col_handle = Some(col_handle);
	}

	pub fn getColHandle(&mut self) -> CollisionObjectHandle {
		self.col_handle.unwrap()
	}

	pub fn removeColHandle(&mut self) {
		self.col_handle = None;
	}

	pub fn pickUpCoin(&mut self) {
		self.pickedup = true;
	}

	pub fn isPickedUp(&mut self) -> bool {
		self.pickedup
	}
}

