use glfw::{Context, Glfw, SwapInterval, Window};

use crate::client::client_settings::ClientSettings;
use crate::client::controller::{ControlHandler, MovementAction};
use crate::client::fps::FpsCounter;
use crate::client::opengl::gl;
use crate::client::opengl::gl::{COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT};
use crate::client::render::world_renderer::WorldRenderer;
use crate::client::viewport::Viewport;
use crate::misc::pos::WorldPos;
use crate::misc::random_quote;
use crate::Player;
use crate::world::{PlayerId, World};

mod opengl;
mod viewport;
mod fps;
mod controller;
mod client_settings;
mod render;
mod input;

pub struct ClientHandler {
	glfw: Glfw,
	window: Window,
	viewport: Viewport,
	fps_counter: FpsCounter,

	settings: ClientSettings,

	player: PlayerId,
	world: Option<World>,
	world_renderer: WorldRenderer,

	control_handler: ControlHandler,
	movement_action: MovementAction,
}

impl ClientHandler {
	// viewport: &Viewport, player: &Player, world: &mut World
	pub fn draw(&mut self) {
		gl::clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
		match &mut self.world {
			None => {}
			Some(world) => {
				let player = world.acquire_player_mut(&self.player);
				self.world_renderer.draw(&self.viewport, player, &self.settings);
			}
		};

		self.window.swap_buffers();

		self.fps_counter.tick();
	}

	pub fn tick(&mut self) {
		self.events();
		match &mut self.world {
			None => {}
			Some(world) => {
				world.tick();
				for x in world.chunk_updates.drain() {
					self.world_renderer.rebuild_chunk(&x);
				}

				self.world_renderer.tick(world, &self.viewport, world.acquire_player(&self.player), &self.settings);
			}
		};
	}

	pub fn join_world(&mut self, mut world: World) {
		let player = Player::new();
		self.player = world.player_join(player);
		self.world = Some(world);
	}

	fn events(&mut self) {
		self.glfw.poll_events();
		self.control_handler.update(|(x, y)| {
			self.viewport.update_size(x, y);
			self.world_renderer.rebuild_all();
		}, |(x, y)| {
			self.settings.zoom = self.settings.zoom - (y as f32 * 4f32)
		});


		match &mut self.world {
			None => {}
			Some(world) => {
				self.movement_action.event_apply(&self.control_handler, world.acquire_player_mut(&self.player));
			}
		};

		self.world_renderer.event_apply(&self.control_handler);
		self.control_handler.finish();
		// let player = &mut self.player;
		// let world = &mut self.world;
		// let mut resize = false;
		// let mut resize_width = 0;
		// let mut resize_height = 0;
		// Box::new(|(x, y)| {
		// 			viewport.update_size(x, y);
		// 			world_renderer.rebuild_all();
		// 		}), Box::new(|(x, y)| {
		// 			settings.zoom = settings.zoom - (y as f32 * 4f32)
		// 		})

		//for (_, event) in glfw::flush_messages(&self.events) {
		//	match event {
		//		WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
		//			self.window.set_should_close(true)
		//		}
		//		WindowEvent::Key(Key::W, _, action, _) => self.control_handler.up.state = !matches!(action, Action::Release),
		//		WindowEvent::Key(Key::A, _, action, _) => self.control_handler.left.state = !matches!(action, Action::Release),
		//		WindowEvent::Key(Key::S, _, action, _) => self.control_handler.down.state = !matches!(action, Action::Release),
		//		WindowEvent::Key(Key::D, _, action, _) => self.control_handler.right.state = !matches!(action, Action::Release),
		//		WindowEvent::Key(Key::Up, _, Action::Press, _) => self.settings.zoom = self.settings.zoom + 10f32,
		//		WindowEvent::Key(Key::Down, _, Action::Press, _) => self.settings.zoom = self.settings.zoom - 10f32,
		//		WindowEvent::Key(Key::C, _, Action::Press, _) => self.settings.chunk_culling = !self.settings.chunk_culling,
		//		WindowEvent::Key(Key::T, _, Action::Press, _) => self.world_renderer.rebuild_all(),
		//		WindowEvent::Key(Key::X, _, Action::Press, _) => self.world_renderer.debug_mode(),
		//		WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
		//			world.map(|mut world| {
		//				let click = self.calc_click();
		//				world.set(&click, Tile::id(tile::STONE));
		//				self.world_renderer.tile_change(&click);
		//			});
		//		}
		//		WindowEvent::MouseButton(MouseButton::Button3, Action::Press, _) => {
		//			world.map(|mut world| {
		//				let click = self.calc_click();
		//				world.set(&click, Wall::id(wall::STONE));
		//				self.world_renderer.tile_change(&click);
		//			});
		//		}
		//		WindowEvent::MouseButton(MouseButton::Button2, Action::Press, _) => {
		//			world.map(|mut world| {
		//				let click = self.calc_click();
		//				world.set(&click, Tile::id(tile::AIR));
		//				self.world_renderer.tile_change(&click);
		//			});
		//		}
		//		WindowEvent::CursorPos(x, y) => {
		//			self.mouse_x = x as f32;
		//			self.mouse_y = y as f32;
		//		}
		//		WindowEvent::Scroll(x, y) => {}
//
		//		WindowEvent::Size(width, height) => {
		//			resize = true;
		//			resize_width = width;
		//			resize_height = height;
		//		}
		//		_ => {}
		//	}
		//}
		//if resize {
		//	self.viewport.update_size(resize_width, resize_height);
		//	self.world_renderer.rebuild_all();
		//}
	}

// pub fn calc_click(&self) -> WorldPos {
// 	let x = self.mouse_x;
// 	let y = self.mouse_y;
// 	let tiles_x = 1f32 / (self.viewport.gl_tile_width / self.settings.zoom);
// 	let tiles_y = 1f32 / (self.viewport.gl_tile_height / self.settings.zoom);
// 	let tile_x = (self.player.pos_x + ((((x / self.viewport.width as f32) * 2f32) - 1f32) * tiles_x)).floor();
// 	let tile_y = (self.player.pos_y - ((((y / self.viewport.height as f32) * 2f32) - 1f32) * tiles_y)).floor();
// 	WorldPos::new(tile_x as i32, tile_y as u32)
// }

	pub fn create() -> ClientHandler {
		println!("Launching GLFW");
		let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();


		println!("Launching Window");
		let title = random_quote::TITLE_QUOTES.get(1).unwrap();
		let (mut window, events) = glfw
			.create_window(900, 600, title, glfw::WindowMode::Windowed)
			.expect("Failed to create GLFW window.");

		window.set_key_polling(true);
		window.set_mouse_button_polling(true);
		window.set_cursor_pos_polling(true);
		window.set_scroll_polling(true);
		window.set_size_polling(true);
		window.make_current();


		println!("Setting GL Context");
		opengl_raw::gll::load_with(|s| glfw.get_proc_address_raw(s));
		opengl_raw::gll::Viewport::load_with(|s| glfw.get_proc_address_raw(s));


		println!("Preparing Graphical Backend");
		let viewport = Viewport::new(900, 600);
		let fps_counter = FpsCounter::new();

		let settings = ClientSettings::new();
		let mut control_handler = controller::ControlHandler::new(events);
		let movement_action = MovementAction::new(&mut control_handler);
		let world_renderer = WorldRenderer::new(&mut control_handler);


//		glfw.set_swap_interval(SwapInterval::Sync(1));
		println!("Finishing");
		gl::clear_color(0.5f32, 0.6f32, 0.98f32, 1f32);
		gl::viewport(0, 0, 900, 600);

		//gl::enable(gl::CULL_FACE);
		//gl::cull_face(gl::BACK);
		//gl::front_face(gl::CW);
		//gl::enable(gl::DEPTH_TEST);

		gl::enable(gl::BLEND);
		gl::blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

		Self {
			glfw,
			window,
			viewport,
			fps_counter,
			settings,
			world: None,
			player: PlayerId::new(),
			world_renderer,
			control_handler,
			movement_action,
		}
	}
}