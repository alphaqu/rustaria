use std::collections::HashSet;
use std::sync::mpsc::Receiver;

use glfw::{Action, Context, Glfw, Key, MouseButton, SwapInterval, Window, WindowEvent};

use crate::{Player};
use crate::client::chunk_renderer::ChunkRenderer;
use crate::client::controller::ControlHandler;
use crate::client::fps::FpsCounter;
use crate::client::opengl::{gl};
use crate::client::opengl::gl::{COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT};
use crate::client::viewport::Viewport;
use crate::world::tile::TileId;
use crate::misc::random_quote;
use crate::pos::{ChunkPos, WorldPos};
use crate::settings::Settings;
use crate::util::CHUNK_SIZE;
use crate::world::{Chunk, tile, wall, World};
use crate::world::neighbor::NeighborAware;
use crate::world::tile::Tile;
use crate::world::wall::Wall;

pub mod chunk_renderer;
pub mod opengl;
pub mod viewport;
pub mod fps;
pub mod controller;
mod client_settings;

pub struct ClientHandler<'a> {
	glfw: Glfw,
	window: Window,
	events: Receiver<(f64, WindowEvent)>,
	viewport: Viewport,
	fps_counter: FpsCounter,

	mouse_x: f32,
	mouse_y: f32,

	world: World<'a>,
	player: Player,


	chunk_renderer: ChunkRenderer,
	control_handler: ControlHandler,
}

impl<'a> ClientHandler<'a> {
	// viewport: &Viewport, player: &Player, world: &mut World
	pub fn draw(&mut self, settings: &mut Settings) {

		gl::clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
		self.chunk_renderer.draw(&self.viewport,  &self.player, settings);
		self.window.swap_buffers();

		self.fps_counter.tick();
	}

	pub fn tick(&mut self, settings: &mut Settings) {
		self.world.tick();
		for x in self.world.chunk_updates.drain() {
			self.chunk_renderer.rebuild_chunk(&x);
		}

		self.events(settings);
		self.chunk_renderer.tick(&self.viewport, &self.player, &mut self.world, settings);
	}

	fn events(&mut self, settings: &mut Settings) {
		let player = &mut self.player;
		let world = &mut self.world;

		self.glfw.poll_events();
		let mut resize = false;
		let mut resize_width = 0;
		let mut resize_height = 0;

		for (_, event) in glfw::flush_messages(&self.events) {
			match event {
				WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
					self.window.set_should_close(true)
				}
				WindowEvent::Key(Key::W, _, action, _) => self.control_handler.up.state = !matches!(action, Action::Release),
				WindowEvent::Key(Key::A, _, action, _) => self.control_handler.left.state = !matches!(action, Action::Release),
				WindowEvent::Key(Key::S, _, action, _) => self.control_handler.down.state = !matches!(action, Action::Release),
				WindowEvent::Key(Key::D, _, action, _) => self.control_handler.right.state = !matches!(action, Action::Release),
				WindowEvent::Key(Key::Up, _, Action::Press, _) => settings.zoom = settings.zoom + 10f32,
				WindowEvent::Key(Key::Down, _, Action::Press, _) => settings.zoom = settings.zoom - 10f32,
				WindowEvent::Key(Key::C, _, Action::Press, _) => settings.cull_chunks = !settings.cull_chunks,
				WindowEvent::Key(Key::T, _, Action::Press, _) => self.chunk_renderer.rebuild_all(),
				WindowEvent::Key(Key::X, _, Action::Press, _) => self.chunk_renderer.debug_mode(),
				WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
					let x = self.mouse_x;
					let y = self.mouse_y;
					let tiles_x = 1f32 / (self.viewport.gl_tile_width / settings.zoom);
					let tiles_y = 1f32 / (self.viewport.gl_tile_height / settings.zoom);
					let tile_x = (player.pos_x + ((((x / self.viewport.width as f32) * 2f32) - 1f32) * tiles_x)).floor();
					let tile_y = (player.pos_y - ((((y / self.viewport.height as f32) * 2f32) - 1f32) * tiles_y)).floor();
					let pos = WorldPos::new(tile_x as i32, tile_y as u32);
					world.set(&pos, Tile::id(tile::AMETHYST));
					self.chunk_renderer.tile_change(&pos);
				},
				WindowEvent::MouseButton(MouseButton::Button3, Action::Press, _) => {
					let x = self.mouse_x;
					let y = self.mouse_y;
					let tiles_x = 1f32 / (self.viewport.gl_tile_width / settings.zoom);
					let tiles_y = 1f32 / (self.viewport.gl_tile_height / settings.zoom);
					let tile_x = (player.pos_x + ((((x / self.viewport.width as f32) * 2f32) - 1f32) * tiles_x)).floor();
					let tile_y = (player.pos_y - ((((y / self.viewport.height as f32) * 2f32) - 1f32) * tiles_y)).floor();
					let pos = WorldPos::new(tile_x as i32, tile_y as u32);
					world.set(&pos, Wall::id(wall::STONE));
					self.chunk_renderer.tile_change(&pos);
				}
				WindowEvent::MouseButton(MouseButton::Button2, Action::Press, _) => {
					let x = self.mouse_x;
					let y = self.mouse_y;
					let tiles_x = 1f32 / (self.viewport.gl_tile_width / settings.zoom);
					let tiles_y = 1f32 / (self.viewport.gl_tile_height / settings.zoom);
					let tile_x = (player.pos_x + ((((x / self.viewport.width as f32) * 2f32) - 1f32) * tiles_x)).floor();
					let tile_y = (player.pos_y - ((((y / self.viewport.height as f32) * 2f32) - 1f32) * tiles_y)).floor();
					let pos = WorldPos::new(tile_x as i32, tile_y as u32);
					world.set(&pos, Tile::id(tile::AIR));
					self.chunk_renderer.tile_change(&pos);
				}
				WindowEvent::CursorPos(x, y) => {
					self.mouse_x = x as f32;
					self.mouse_y = y as f32;
				}
				WindowEvent::Scroll(x, y) => {
					settings.zoom = settings.zoom - (y as f32 * 4f32)
				}

				WindowEvent::Size(width, height) => {
					resize = true;
					resize_width = width;
					resize_height = height;
				}
				_ => {}
			}
		}

		self.control_handler.tick(player);
		if resize {
			self.viewport.update_size(resize_width, resize_height);
			self.chunk_renderer.rebuild_all();
		}
	}

	pub fn launch(player: &'a Player) -> ClientHandler<'a> {
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
		glfw.set_swap_interval(SwapInterval::Sync(1));
		let mut viewport = Viewport::new(900, 600);
		let mut fps_counter = FpsCounter::new();


		println!("Preparing Chunk Renderer");
		let chunk_renderer = ChunkRenderer::new(player);


		println!("Finishing");
		gl::clear_color(0.5f32, 0.6f32, 0.98f32, 1f32);
		gl::viewport(0, 0, 900, 600);

		//gl::enable(gl::CULL_FACE);
		//gl::cull_face(gl::BACK);
		//gl::front_face(gl::CW);
		//gl::enable(gl::DEPTH_TEST);

		gl::enable(gl::BLEND);
		gl::blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);


		let mut handler = Self {
			glfw,
			window,
			events,
			viewport,
			fps_counter,
			mouse_x: 0.0,
			mouse_y: 0.0,
			world: World::new(),
			player: Player::new(),
			chunk_renderer,
			control_handler: ControlHandler::new(),
		};
		handler
	}
}
