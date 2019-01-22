//! `Player` contains all of the methods for the player character movements,
//! collisions, and other functions needed to interact with the game world.

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
use actors::step_queue::{StepQueue, Step};
use ncollide::world::{CollisionObjectHandle, CollisionWorld2};


/// Constants used for movement physics.
pub const STEP_PERIOD: f64 = 1.0 / 60.0;
const MOVE_ACCEL: f64 = 150. * STEP_PERIOD;
const STOP_ACCEL: f64 = 350. * STEP_PERIOD;
const FALL_ACCEL: f64 = 360. * STEP_PERIOD;
const JUMP_SPEED: f64 = 60.;
const MAX_FALL_SPEED: f64 = 40.;
const MAX_MOVE_SPEED: f64 = 10.;
fn jump_duration(x_speed: f64) -> f64 { 0.21 + 0.10 * (x_speed / MAX_MOVE_SPEED) } 

const GRAPHIC_STEP_DURATION: f64 = 0.16;


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

/// `Time` will be used to maintain event timing. This will be properly implemented
/// in the future. I found that not having a time feature was causing some graphics
/// and physics based anomalies, and found resources that explained how timing can
/// help smooth out user input and relevant actions.
struct Time{
	time: f64,
}

/// `PlayerState` is an enumerated value of the allowable player states
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum PlayerState{
	Jumping,
	Walking,
	Idle,
}

///
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum PlayerSize {
	Big,
}

#[derive(Clone, Copy)]
pub struct Player {
	time: f64,
	state_start_time: f64,
	pub tag: ActorType,
	pub pos: Vector2,
	dir: Direction,
	pub currentState: PlayerState,
	size: PlayerSize,
	moving: bool,
	pub grounded: bool,
	jump_time: f64,
	pub velocity: Vector2,
	//rec: SpriteRectangle,
	debug: bool,
	col_handle: Option<CollisionObjectHandle>,
	step_queue: StepQueue,
}


impl Player {
	pub fn new(pos: Vector2, time: f64, move_dir: Option<Direction>) -> Player {
		let mut player = Player {
			time,
			state_start_time: time,
			tag: ActorType::Player,
			pos: Vector2::new(-1920., 0.),
			dir: Direction::Right,
			currentState: PlayerState::Jumping,
			size: PlayerSize:: Big,
			moving: false,
			grounded: false,
			jump_time: time,
			velocity: na::zero(),
			debug: true,
			col_handle: None,
			step_queue: StepQueue::new(),
		};
		player.set_movement(move_dir);
		(player)
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

	/// `unput()` is a function for the `InputEvent` handler.
	pub fn input(&mut self, event:InputEvent) {
		match event {
			InputEvent::UpdateMovement(Some(direction)) => self.dir = direction,
			_ => {},
		}
		match event {
			// No movement update (used for key up)
			InputEvent::UpdateMovement(None) => {
					self.currentState = PlayerState::Idle;
					self.set_movement((None));
					self.moving = false;
					self.step_queue.peek_specific(Step::Player);
			}
			// Player is moving in a direction
			InputEvent::UpdateMovement(direction) =>{
				if self.currentState == PlayerState::Jumping && !self.grounded{
					self.set_movement(direction);
					self.step_queue.peek_specific(Step::Player);
				}
				else {
					self.currentState = PlayerState::Walking;
					self.set_movement((direction));
					self.step_queue.peek_specific(Step::Player);
				}
			} 
			// Player pressed Jump
			InputEvent::PressJump => {
				if self.currentState != PlayerState::Jumping{
					self.currentState = PlayerState::Jumping;
					self.jump();
					self.advance();
				}
			}
			// Initially used for timed updates, but was causing issues. Kept in case
			// it is needed for future implementation
			InputEvent::TimeUpdate => {
				// currently not used
			}
			// Player landed on the ground (occurs during collision event with ground)
			InputEvent::Landed => {
				if !self.grounded{
					self.velocity.y = 0.;
					self.grounded = true;;
					let direction = self.dir;
					if self.moving {
						self.currentState = PlayerState::Walking;
						self.set_movement(Some(direction));
					}
					else {
						self.currentState = PlayerState::Idle;
						self.set_movement(None);
					}
					self.step_queue.peek_specific(Step::Player);
				}
			}

		}
	}

	/// `advance()` utilizes the `StepQueue` data stucture which maintains a list of movements.
	/// The `StepQueue` utilizes a push/pop to help maintain proper ordering of the movements.
	pub fn advance(&mut self) {
			match self.step_queue.pop() {
				Step::Player => self.step(),
				_ => {},
			}
	}

	/// `set_movement()` calculates if and how a player is moving (jumping, walking).
    pub fn set_movement(&mut self, movement: Option<Direction>) {

    	if !self.grounded && self.currentState != PlayerState::Jumping {
    		self.currentState = PlayerState::Jumping;
    	}

    	if self.currentState == PlayerState::Jumping || self.currentState == PlayerState::Walking {
	        let (moving, dir) = if let Some(dir) = movement { (true, dir) } else { (false, self.dir) };
	        if self.moving != moving || self.dir != dir { self.state_start_time = self.time; }
	        self.moving = moving;
	        self.dir = dir;
    	}
    	else {
    		self.moving = false;
    	}
    }

	/// `step()` calculates the velocity of the player character and updates the new position
	/// based on the velocity and the current (previous) location of the player character.
    pub fn step(&mut self) {
    	let stop_accel = if self.grounded {STOP_ACCEL} else { MOVE_ACCEL };
    	let rel_vel_x = if self.velocity.x != na::zero() {self.velocity.x} else { 0.0 };
    	let rel_vel_x = if self.moving {
    	let accel = if self.dir.movement() == ((rel_vel_x as f64).signum()) { MOVE_ACCEL } else { stop_accel };
    	rel_vel_x + (self.dir.movement() * accel) as f32
    	}
    	else if self.grounded {
    	if rel_vel_x.abs() > ((stop_accel) as f32) {rel_vel_x - ((rel_vel_x).signum()) * (stop_accel as f32) } else { 0.0 }
    	}
    	else {
    		rel_vel_x
    	};
    	self.velocity.x = rel_vel_x;

    	if self.time > self.jump_time || !self.grounded {
    		self.velocity.y -= FALL_ACCEL as f32;
    	}
       	self.pos.x = self.pos.x + self.velocity.x;   
        self.pos.y = self.pos.y + self.velocity.y;

    	self.update_movement();
    	self.update_grounded(false);
    }

    /// `jump()` calculates the player jump velocity and direction (if any).
    fn jump(&mut self) {
    	if self.grounded { 
	    	self.grounded = false;
	    	self.state_start_time = self.time;
	    	self.jump_time = self.time + jump_duration((self.velocity.x).abs() as f64);
	    	self.velocity.y = JUMP_SPEED as f32;

	    	let direction = self.dir;
					if self.moving {
						self.set_movement(Some(direction));
					}
					else {
						self.set_movement(None);
					}
			self.step_queue.peek_specific(Step::Player);
    	}

    }

    /// `update_movement()` ensures the vertical and horizontal velocity of the player character
    /// doesn't exceed the `MAX_MOVE_SPEED` and `MAX_FALL_SPEED` restrictions.
    /// Since we are modifying the coordinate system of the game for everything originating from
    /// the top left pixel, Y axis increases as it goes down, so we inverted the fall speed.
    fn update_movement(&mut self) {
    	self.velocity.x = self.velocity.x.max(-MAX_MOVE_SPEED as f32);

    	self.velocity.x = self.velocity.x.min(MAX_MOVE_SPEED as f32);

    	if self.currentState == PlayerState::Jumping{
    		self.velocity.y = self.velocity.y.max(-MAX_FALL_SPEED as f32);
    	}
    	else {
    		self.velocity.y = 0.;
    	}
    }

    /// `update_grounded` calculates if a character is grounded based off its current player
    /// state and other attributes. If the player is grounded, we will update the movement
    /// to ensure there is no further vertical movement as if falling.
    fn update_grounded(&mut self, near_ground: bool) {
    	let on_ground = self.velocity.y == 0.0 && self.currentState != PlayerState::Jumping;
    	match(self.grounded, on_ground, near_ground) {
    		(false, true, _) => {
    			self.state_start_time = (self.time as f64) - GRAPHIC_STEP_DURATION;
    			self.grounded = true;
    			self.update_movement();
    		},
    		(true, false, false) => {
    			self.state_start_time = self.time;
    			self.grounded = false;
    			self.update_movement();
    		},
    		(true, false, true) => {
    			self.velocity.y = -MAX_FALL_SPEED as f32;
    		},
    		_ => {},
    	}
    }
}