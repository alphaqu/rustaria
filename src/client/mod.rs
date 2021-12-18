use std::sync::mpsc::Receiver;

use glfw::{Action, Context, Glfw, Key, SwapInterval, Window, WindowEvent};

use crate::{gll, Player};
use crate::client::chunk_renderer::ChunkRenderer;
use crate::client::controller::ControlHandler;
use crate::client::fps::FpsCounter;
use crate::client::opengl::gl;
use crate::client::opengl::gl::{COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT};
use crate::client::viewport::Viewport;
use crate::settings::Settings;
use crate::world::World;

pub mod chunk_renderer;
pub mod opengl;
pub mod viewport;
pub mod fps;
pub mod controller;

pub struct ClientHandler {
    glfw: Glfw,
    window: Window,
    events: Receiver<(f64, WindowEvent)>,
    viewport: Viewport,
    fps_counter: FpsCounter,

    chunk_renderer: ChunkRenderer,
    control_handler: ControlHandler,
}

impl ClientHandler {
    // viewport: &Viewport, player: &Player, world: &mut World
    pub fn draw(&mut self, world: &mut World, player: &mut Player, settings: &mut Settings) {
        gl::clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);

        self.chunk_renderer.tick(&self.viewport, player, world, settings);
        self.chunk_renderer.draw(&self.viewport, player, settings);
        self.window.swap_buffers();

        self.fps_counter.tick();
    }

    pub fn tick(&mut self, world: &mut World, player: &mut Player, settings: &mut Settings) {
        self.events(world, player, settings);
    }

    fn events(&mut self, world: &mut World, player: &mut Player, settings: &mut Settings) {
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
            self.chunk_renderer.rebuild(&self.viewport, player, world, settings);
        }
    }

    pub fn launch(world: &mut World, player: &Player, settings: &Settings) -> ClientHandler {
        println!("Launching GLFW");
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();


        println!("Launching Window");
        let (mut window, events) = glfw
            .create_window(900, 600, "Hello this is window", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");

        window.set_key_polling(true);
        window.set_scroll_polling(true);
        window.set_size_polling(true);
        window.make_current();


        println!("Setting GL Context");
        gll::load_with(|s| glfw.get_proc_address_raw(s));
        gll::Viewport::load_with(|s| glfw.get_proc_address_raw(s));


        println!("Preparing Graphical Backend");
        glfw.set_swap_interval(SwapInterval::Sync(1));
        let mut viewport = Viewport::new(900, 600);
        let mut fps_counter = FpsCounter::new();


        println!("Preparing Chunk Renderer");
        let chunk_renderer = ChunkRenderer::new(player);


        println!("Finishing");
        gl::clear_color(0.5f32, 0.6f32, 0.98f32, 1f32);
        gl::viewport(0, 0, 900, 600);
        gl::enable(gl::CULL_FACE);
        let mut handler = Self {
            glfw,
            window,
            events,
            viewport,
            fps_counter,
            chunk_renderer,
            control_handler: ControlHandler::new(),
        };

        &handler.chunk_renderer.tick(&handler.viewport, player, world, settings);
        handler
    }
}
