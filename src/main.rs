//! A 2D platform-style game example featuring our hero Ferris
//! trying to get a safe rusty drink from a vending machine.
//! Unforunately our friend Ferris has had his money stolen from
//! his unsafe brother, Neferrious.
//! Will Ferris be able to quench his thrist?

#[macro_use]
extern crate ggez;
extern crate nalgebra;
extern crate ncollide;

pub mod actors;
mod game_inputs;

use std::env;
use std::path;
use std::collections::{BTreeMap, LinkedList};
use std::time::{Duration, Instant};
use std::str::FromStr;
use std::f32;
use std::ops::Mul;
use ggez::audio;
use ggez::conf;
use ggez::conf::FullscreenType;
use ggez::event::{self, EventHandler, Keycode, Mod};
use ggez::graphics;
use ggez::graphics::{Color, DrawMode, Rect, Vector2, Point2, rectangle, set_color,
                      DrawParam, TextCached, TextFragment, Scale, HorizontalAlign as HAlign, Layout};
use ggez::{Context, ContextBuilder, GameResult};
use ggez::nalgebra as na;
use ggez::nalgebra::{Isometry2};
use ggez::timer;
use actors::player::Player;
use actors::coin::Coin;
use actors::object::Object;
use actors::types::{ActorType, CollisionObjectData};
use game_inputs::{Direction, GameInput, InputEvent};
use ncollide::shape::{Cuboid2, ShapeHandle2};
use ncollide::procedural::circle;
use ncollide::world::{CollisionGroups, CollisionObjectHandle, CollisionWorld2, GeometricQueryType};
use ncollide::events::{ContactEvent, ProximityEvent};
use ncollide::query::Proximity;
use ncollide::narrow_phase::ContactAlgorithm;


/// Basic information including desired window size for Context window_mode.
/// This isn't directly utilizied as I have migrated to forcing full screen;
/// however, I will have to add additional functionality to scale the image
/// in the case the native resolution isn't 1080 x 1920 (the default image
/// resolution)
/// Additional standard measurements for the player png are listed.
const WINDOW_HEIGHT:f32 = 1080.;
const WINDOW_WIDTH:f32 = 1920.;
const FERRIS_HEIGHT:f32 = 167.;
const FERRIS_WIDTH:f32 = 226.;

/// ***************************************************************************
/// # Assets
/// 'Assets' contain the various game assets such as text font, music, sounds,
/// player icon, etc... These objects are all hard coded.
/// ***************************************************************************

struct Assets {
	player_image: graphics::Image,
	coin_image: graphics::Image,
    vending_image: graphics::Image,
	font: graphics::Font,
	main_music: audio::Source,
    end_music: audio::Source,
	jump: audio::Source,
	coin_jingle: audio::Source,
}

impl Assets {
	fn new(ctx: &mut Context) -> GameResult<Assets> {
		let player_image = graphics::Image::new(ctx, "/player.png")?;
		let coin_image = graphics::Image::new(ctx, "/coin.png")?;
        let vending_image = graphics::Image::new(ctx, "/vendingMachine.png")?;
		let font = graphics::Font::new(ctx, "/prstartk.ttf", 32)?;
		let main_music = audio::Source::new(ctx, "/Rolemusic_-_07_-_Beach_Wedding_Dance.ogg")?;
        let end_music= audio::Source::new(ctx, "/Rolemusic_-_neogauge.ogg")?;
		let jump = audio::Source::new(ctx, "/jump.wav")?;
		let coin_jingle = audio::Source::new(ctx, "/coin_jingle.ogg")?;
		Ok(Assets {player_image, coin_image, vending_image, font, main_music, end_music, jump, coin_jingle})
	}

	fn actor_image(&mut self) -> &mut graphics::Image {
			&mut self.player_image
	}

	fn coin_image(&mut self) -> &mut graphics::Image {
		&mut self.coin_image
	}

    fn vending_image(&mut self) -> &mut graphics::Image {
        &mut self.vending_image
    }
}




/// Borrowed from GGEZ Astroblasto example
/// This is used to transalte the world coordinate system which has both Y == 0
/// and X == 0 being the origin (center of the screen), and converts it to the
/// screen coordinate system which has the origin in the upper left of the
/// screen with Y inverted (increasing in a downward direction).
/// This helps with converting all items being rendred from the top-left.
fn world_to_screen_coords(screen_width: u32, screen_height: u32, point: Vector2) -> Vector2 {
    let width = WINDOW_WIDTH as f32;
    let height = WINDOW_HEIGHT as f32;
    let x = point.x + width / 2.0;
    let y = height - (point.y + height / 2.0);
    Vector2::new(x, y)
}

/// A function used to draw the actor graphic at its current position. This
/// position is helped by the world_to_screen_coords() method listed earlier.
fn draw_actor(
	assets: &mut Assets,
	ctx: &mut Context,
	player: &Player,
	world_coords: (u32, u32),) -> GameResult<()> {

	let (screen_w, screen_h) = world_coords;
	let pos = world_to_screen_coords(screen_w, screen_h, player.pos);
	let image = assets.actor_image();
	let drawparams = graphics::DrawParam {
		dest: Point2::new(pos.x, pos.y),
		offset: graphics::Point2::new(0.0, 0.0),
		..Default::default()
	};

	graphics::draw_ex(ctx, image, drawparams)
}

/// A function used to draw the coin graphic at its current position. This
/// position is helped by the world_to_screen_coords() method listed earlier.
fn draw_coin(
	assets: &mut Assets,
	ctx: &mut Context,
	coin: &Coin,
	world_coords: (u32, u32),) -> GameResult<()> {

	let (screen_w, screen_h) = world_coords;
	let pos = world_to_screen_coords(screen_w, screen_h, coin.pos);
	let image = assets.coin_image();
	let drawparams = graphics::DrawParam {
		dest: Point2::new(pos.x, pos.y),
		offset: graphics::Point2::new(0.0, 0.0),
		..Default::default()
	};

	graphics::draw_ex(ctx, image, drawparams)
}

/// ***************************************************************************
/// A function used to draw the vending machine object graphic at its current
/// position. This position is helped by the world_to_screen_coords() method
/// listed earlier.
/// ***************************************************************************
fn draw_vending(
    assets: &mut Assets,
    ctx: &mut Context,
    vending: &mut Object,
    world_coords: (u32, u32),) -> GameResult<()> {
    let (screen_w, screen_h) = world_coords;
    let pos = world_to_screen_coords(screen_w, screen_h, vending.pos);
    let image = assets.vending_image();
    let drawparams = graphics::DrawParam {
        dest: Point2::new(pos.x, pos.y),
        offset: graphics::Point2::new(0.0, 0.0),
        ..Default::default()
    };

    graphics::draw_ex(ctx, image, drawparams)
}




/// # Contact handler
///
/// `handle_contact_event()` is used a collision event handler used to assist
/// the collision events of the player with the ground, coin, and vending
/// machine. This will be expanded to help with collisions of various ground
/// and coin objects (required to help with more expansive level objects).
fn handle_contact_event(player: &mut Player, coin: &mut Coin,  vending: &mut Object, world: &CollisionWorld2<f32, ()>, assets: &Assets, event: &ContactEvent, ctx: &mut Context) -> i32 {
	let mut s = 0;
    if let &ContactEvent::Started(collider1, collider2) = event {

    	let co1 = world.collision_object(collider1).unwrap();
    	let co2 = world.collision_object(collider2).unwrap();
        // check if collision object is coin
    	if co1.handle() == coin.getColHandle() || co2.handle() == coin.getColHandle() {
    		if !coin.isPickedUp(){
    			coin.pickUpCoin();
    			println!("Picked up coin?: {:?}", coin.isPickedUp());
    			s = 1337;
    			let _ = assets.coin_jingle.play();
    		}
    	}
        // check if collision object is the vending machine
        else if co1.handle() == vending.getColHandle() || co2.handle() == vending.getColHandle() {
            if !vending.isPickedUp(){
                vending.pickUpObject();
                let _ = assets.main_music.stop();
                let _ = assets.end_music.play();
                println!("Vending Machine Reached?: {:?}", vending.isPickedUp());
            }
        }
        // else vending machine is the ground
    	else{
    		if player.grounded == false {
				player.input(InputEvent::Landed);
			}
	    	let vector = co1.position().translation.vector.data;

			let pos = world_to_screen_coords(ctx.conf.window_mode.height, ctx.conf.window_mode.width, Vector2::new(co1.position().translation.vector.data[0], co1.position().translation.vector.data[1]));

			player.pos.y = pos.y + FERRIS_HEIGHT +50.;
		}
    }
    // return int s: either 0 or x>0 (picked up coin)
    s
}

/// # MainState
/// `MainState` is a structure used to contain the games current state. Various
/// states can be used in the future to assist with different levels, menus,
/// completion, and other states as neccessary. The main will need to implement
/// a list of states instead of calling a single MainState.
struct MainState {
	image1: graphics::Image,
    text: graphics::Text,
    frames: usize,
    assets: Assets,
    player: Player,
    coin: Coin,
    vending: Object,
    score: i32,
    score_display: graphics::Text,
    win_bool: bool,
    win_display: BTreeMap<&'static str, TextCached>,
    screen_width: u32,
    screen_height: u32,
    gameInput: GameInput,
    world: CollisionWorld2<f32, ()>,

}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        // The ttf file will be in the resources directory. Later, we
        // will mount that directory so we can omit it
        // in the path here.

        let assets = Assets::new(ctx)?;
        let text = graphics::Text::new(ctx, "Hello Ferris!", &assets.font)?;
        let score_disp = graphics::Text::new(ctx, "Score:", &assets.font)?;
        let mut win_disp = BTreeMap::new();
        let image1 = graphics::Image::new(ctx, "/beach.png")?;
        graphics::set_background_color(ctx, (0, 0, 0, 255).into());

        let pos = world_to_screen_coords(ctx.conf.window_mode.height, ctx.conf.window_mode.width, Vector2::new(-1920., 0.));

        let mut player = actors::player::Player::new(pos, 1.0, Some(Direction::Right));
        let mut coin = actors::coin::Coin::new(pos);
        let mut vending = actors::object::Object::new(Vector2::new(1920./2. - 275., -1080./4. + 350.));
        let _ = assets.main_music.play();
        // set MainState
        let mut s = MainState {
        	image1,
        	text,
        	frames: 0,
        	assets,
        	player,
        	coin,
            vending,
        	score: 0,
        	score_display: score_disp,
            win_bool: false,
            win_display: win_disp,
        	screen_width: ctx.conf.window_mode.width,
        	screen_height: ctx.conf.window_mode.height,
        	gameInput: GameInput::new(),
        	world: CollisionWorld2::new(0.02),
        };
        /// modify score value to default
        let score_str = format!("Score: {}", 0);
        let score_text = graphics::Text::new(ctx, &score_str, &s.assets.font).unwrap();
        s.score_display = score_text;
        // modify & set win message
        let test_font = graphics::Font::new_glyph_font(ctx, "/prstartk.ttf")?;
        let mut text = TextCached::new_empty()?;
        text.add_fragment("Congratulations!!  ");
        text.add_fragment("You have helped Ferris find a coin, ");
        text.add_fragment("and quench his thirst for Safety!  ");
        text.add_fragment("Press Escape at any time to exit.");
        text.set_font(test_font.clone(), Scale::uniform(40.0))
            .set_bounds(
                Point2::new(1000.0, 1000.0),
                Some(Layout::default().h_align(HAlign::Center)),
            );
        s.win_display.insert("Win_Message", text.clone());
                
        Ok(s)
    }

    // Add collision object to the current state. This is used to add player
    // and environmental objects for future potential collision handling.
    pub fn add_collision_entity(&mut self, isometry: Isometry2<f32>, shape_handle: ShapeHandle2<f32>, groups: CollisionGroups, query: GeometricQueryType<f32>) -> CollisionObjectHandle {
		self.world.add(isometry, shape_handle, groups, query, ())
	}

    // Update the score display text
	fn update_ui(&mut self, ctx: &mut Context) {
		let score_str = format!("Score: {}", self.score);
		let score_text = graphics::Text::new(ctx, &score_str, &self.assets.font).unwrap();
		self.score_display = score_text;

	}
}

// Then we implement the `ggez:event::EventHandler` trait on it, which
// requires callbacks for updating and drawing the game state each frame.
//
// The `EventHandler` trait also contains callbacks for event handling
// that you can override if you wish, but the defaults are fine.
impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        // `DESIRED_FPS` is used to modify the `timer` object calls. This will have a
        // direct impact on the FPS; however, in the current state, game physics is
        // directly tied to the frame rate. In order to modify this, we will need to
        // make the objects movement and other calls independent of the frame rate.
        // This will be done by implementing `time` within the object methods. This
        // timer will offset the forced time updates of a manipulated frame update call.
    	const DESIRED_FPS: u32 = 60;

    	while timer::check_update_time( _ctx, DESIRED_FPS) {

        	if (self.player.pos.x > (WINDOW_WIDTH / 2.) - 226.) || (self.player.pos.x < - (WINDOW_WIDTH /2.)) {
        		if self.player.pos.x > (WINDOW_WIDTH / 2.) - 226. {
    				self.player.pos.x = (WINDOW_WIDTH / 2.) - 227.;
    			}
    			else {
    				self.player.pos.x = -WINDOW_WIDTH / 2. + 5.;
    			}
        		self.player.velocity.x = 0.;
        		self.player.input(InputEvent::UpdateMovement(None));
        	}
        	self.player.advance();
        	self.player.update(_ctx, &mut self.world);

        	if !self.coin.isPickedUp(){
        		self.coin.update(_ctx, &mut self.world);
        	}

            self.vending.update(_ctx, &mut self.world);

        	self.world.update();
        	
        	if self.world.contacts().count() > 0 {
        		
    			for event in self.world.contact_events() {
        			let s = handle_contact_event(&mut self.player, &mut self.coin, &mut self.vending, &self.world, &self.assets, event, _ctx);

        			self.score = self.score + s;
        			
                    if self.vending.isPickedUp() {
                        self.win_bool = true;
                    }

    			}

    		}
		}	
        Ok(())
    }

    // A function that is consistently called to draw various assets on the screen.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        // Update Scoreboard
        if format!("Score: {}", self.score) != self.score_display.contents() {
            self.update_ui(ctx);
        }

        let coords = (self.screen_width, self.screen_height);

        let assets = &mut self.assets;
        let p = &self.player;
        let dst = graphics::Point2::new(0.0, 0.0);
        graphics::draw(ctx, &self.image1, dst, 0.0)?;

        draw_vending(assets, ctx, &mut self.vending, coords)?;

        if !self.coin.isPickedUp(){
			draw_coin(assets, ctx, &self.coin, coords)?;
		}
        draw_actor(assets, ctx, p, coords)?;

		let pos = world_to_screen_coords(ctx.conf.window_mode.height, ctx.conf.window_mode.width, Vector2::new(0., 0.));

        // Drawables are drawn from their top-left corner.
        let dest_point = graphics::Point2::new(10.0, 10.0);
        let score_point = graphics::Point2::new(10.0, 80.0);

        graphics::draw_ex(
                ctx,
                &self.text,
                graphics::DrawParam {
                    dest: graphics::Point2::new(dest_point.x +3., dest_point.y + 3.),
                    color: Some(graphics::Color::from((146, 32, 27, 255))),
                    ..Default::default()
                },
            )?;
        graphics::draw_ex(
                ctx,
                &self.text,
                graphics::DrawParam {
                    dest: dest_point,
                    color: Some(graphics::Color::from((228, 55, 23, 255))),
                    ..Default::default()
                },
            )?;

        graphics::draw_ex(
                ctx,
                &self.score_display,
                graphics::DrawParam {
                	dest: graphics::Point2::new(score_point.x + 3., score_point.y + 3.),
                    color: Some(graphics::Color::from((146, 32, 27, 255))),
                    ..Default::default()
                },
            )?;

        graphics::draw_ex(
                ctx,
                &self.score_display,
                graphics::DrawParam {
                    dest: score_point,
                    color: Some(graphics::Color::from((228, 55, 23, 255))),
                    ..Default::default()
                },
            )?;

        if self.win_bool {
            let mut height = 0.0;
            let background_text = &self.win_display;

            for (_key, text) in background_text {
                let h = text.height(ctx) as f32;
                let w = text.width(ctx) as f32;
                text.queue(ctx, Point2::new(1920./2. - w / 2. +3.,1080./2. - h + height+3. ), Some(graphics::Color::from((0, 0, 0, 255))));
                height += 50. + h;
            }
            height = 0.0;
            for (_key, text) in &self.win_display {
                let h = text.height(ctx) as f32;
                let w = text.width(ctx) as f32;
                text.queue(ctx, Point2::new(1920./2. - w / 2., 1080./2. - h + height), Some(graphics::Color::from((185, 30, 1, 255))));
                height += 50. + h;
            }

            TextCached::draw_queued(ctx, DrawParam::default())?;
        }

        graphics::present(ctx);

        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {}", ggez::timer::get_fps(ctx));
        }
        Ok(())
    }

    /// A function used to handle the keydown events. Will be updated to allow for a list of key
    /// events with times that will be used to help with ensuring key times match up for executing
    /// character interactions.
    #[inline]
    fn key_down_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
    	if let Some(event) = self.gameInput.key_down_event(keycode) {
    		match keycode {
	    		Keycode::Right => {
	    			self.player.input(InputEvent::UpdateMovement(Some(Direction::Right)));
	    		}
	    		Keycode::Left => {
	    			self.player.input(InputEvent::UpdateMovement(Some(Direction::Left)));
	    		}
	    		Keycode::Space => {

	    			if self.player.grounded {
    					let _ = self.assets.jump.play();
	    			}
	    			self.player.input(InputEvent::PressJump);

	    		}
	    		Keycode::Escape => ctx.quit().unwrap(),
	    		_ => {},
    		}
    	}
    	else if keycode == Keycode::Escape { ctx.quit().unwrap(); }
    }
    /// A function used to handle the finishing of a key being pressed down. This will help with
    /// game physics impacts on the main player character.
	#[inline]
    fn key_up_event(&mut self, ctx:&mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
    	if let Some(event) = self.gameInput.key_up_event(keycode) {
	    	match keycode {
	    		Keycode::Right => {
	    			self.player.input(InputEvent::UpdateMovement(None));
	    		}
	    		Keycode::Left => {
	    			self.player.input(InputEvent::UpdateMovement(None));
	    		}
	    		_ => {},
	    	}
    	}
    }
}

/// Now our main function, which does three things:
///
/// * First, create a new `ggez::conf::Conf`
/// object which contains configuration info on things such
/// as screen resolution and window title.
/// * Second, create a `ggez::game::Game` object which will
/// do the work of creating our MainState and running our game.
/// * Then, just call `game.run()` which runs the `Game` mainloop.
pub fn main() {
    let mut cb = ContextBuilder::new("Hello Ferris", "ggez")
    	.window_setup(conf::WindowSetup::default().title("Ferris and the Safe World!"))
        .window_mode(conf::WindowMode{
            width: 1920,
            height: 1080,
            borderless: false,
            fullscreen_type: FullscreenType::True,
            vsync: true,
            min_width: 0,
            max_width: 1920,
            min_height: 0,
            max_height: 1080,
        });

    // We add the CARGO_MANIFEST_DIR/resources to the filesystem's path
    // so that ggez will look in our cargo project directory for files.
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        cb = cb.add_resource_path(path);
    }

    let ctx = &mut cb.build().unwrap();

    let mut state = MainState::new(ctx).unwrap();

    // Create the object shapes to use for our collision handles
    let ground = ShapeHandle2::new(Cuboid2::new(Vector2::new(1920., 32.)));
    let playerShape = ShapeHandle2::new(Cuboid2::new(Vector2::new(220., 160. )));
    let coinShape = ShapeHandle2::new(Cuboid2::new(Vector2::new(0.1, 0.1)));
    let vendShape = ShapeHandle2::new(Cuboid2::new(Vector2::new(200., 450.)));
	let groups = CollisionGroups::new();
	let query = GeometricQueryType::Contacts(0., 0.);
	let pos = world_to_screen_coords(ctx.conf.window_mode.height, ctx.conf.window_mode.width, Vector2::new(0., -500.));
    // Add the ground collision object
	state.add_collision_entity(Isometry2::new(Vector2::new(pos.x -(WINDOW_WIDTH /2.), pos.y), 0.), ground.clone(), groups, query);
    
    let pos = world_to_screen_coords(ctx.conf.window_mode.height, ctx.conf.window_mode.width, Vector2::new(0., 0.));
    // Set the player, coin, and vending machine collision handles
	let player_collision_handle = state.add_collision_entity(Isometry2::new(Vector2::new(pos.x, pos.y), 0.), playerShape.clone(), groups, query);
	let coin_collision_handle = state.add_collision_entity(Isometry2::new(Vector2::new(pos.x, pos.y), 0.), coinShape.clone(), groups, query);
    let vending_collision_handle = state.add_collision_entity(Isometry2::new(Vector2::new(pos.x, pos.y), 0.), vendShape.clone(), groups, query);

    // Add the collision handles to their respective player, coin, and vending machine `Actor` objects.
    state.player.set_col_handle(player_collision_handle);	
    state.coin.set_col_handle(coin_collision_handle);
    state.vending.set_col_handle(vending_collision_handle);



    if let Err(e) = event::run(ctx, &mut state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
