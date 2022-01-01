use glfw::{Context, Glfw, OpenGlProfileHint, Window, WindowHint};

use crate::client::client_settings::ClientSettings;
use crate::client::controller::{ControlHandler, MovementAction};
use crate::client::fps::FpsCounter;
use crate::client::opengl::gl;
use crate::client::opengl::gl::{COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT};
use crate::client::render::world_renderer::WorldRenderer;
use crate::client::viewport::Viewport;
use crate::misc::random_quote;
use crate::Player;
use crate::world::{PlayerId, World};

mod opengl;
mod viewport;
mod fps;
mod controller;
mod client_settings;
mod render;

pub struct ClientHandler {
    glfw: Glfw,
    window: Window,
    viewport: Viewport,

    settings: ClientSettings,

    player_id: PlayerId,
    world: Option<World>,
    world_renderer: WorldRenderer,

    control_handler: ControlHandler,
    movement_action: MovementAction,
}

impl ClientHandler  {
    // viewport: &Viewport, player: &Player, world: &mut World
    #[profiler_macro::profile]
    pub fn draw(&mut self) {
        gl::clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
        match &mut self.world {
            None => {}
            Some(world) => {
                for x in world.chunk_updates.drain() {
                    self.world_renderer.rebuild_chunk(&x);
                }

                let player = world.acquire_player(&self.player_id);
                self.world_renderer.draw(&self.viewport, player, &self.settings);
            }
        };
        self.window.swap_buffers();
    }

    #[profiler_macro::profile]
    pub fn input_tick(&mut self) {
        self.events();
    }

    #[profiler_macro::profile]
    pub fn tick(&mut self) {
        match &mut self.world {
            None => {}
            Some(world) => {
                world.tick_world();
                self.world_renderer.tick_world_renderer(world, &self.viewport, world.acquire_player(&self.player_id), &self.settings);
            }
        };
    }

    pub fn join_world(&mut self, mut world: World) {
        let player = Player::new();
        self.player_id = world.player_join(player);
        self.world = Some(world);
    }

    fn events(&mut self) {
        self.glfw.poll_events();
        self.control_handler.update(|(x, y)| {
            self.viewport.update_size(x, y);
            self.world_renderer.rebuild_all();
        }, |(_x, y)| {
            self.settings.zoom -= (y as f32) / 10f32
        });


        match &mut self.world {
            None => {}
            Some(world) => {
                self.movement_action.event_apply(&self.control_handler, world.acquire_player_mut(&self.player_id));
            }
        };

        self.world_renderer.event_apply(&self.control_handler);
        self.control_handler.finish();
    }

    pub fn create() -> ClientHandler  {
        println!("Launching GLFW");
        let mut glfw = glfw::init(glfw::LOG_ERRORS).unwrap();
        glfw.window_hint(WindowHint::ContextVersionMajor(4));
        glfw.window_hint(WindowHint::ContextVersionMinor(5));
        glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));

        println!("Launching Window");
        let title = random_quote::TITLE_QUOTES.get(1).unwrap();
        let (mut window, events) = glfw
            .create_window(900, 600, title, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");
        println!("Open gl debug context {}", window.is_opengl_debug_context());
        println!("Open gl debug context {:?}", window.get_context_version());
        println!("Open gl compat {}", window.is_opengl_forward_compat());
        window.make_current();
        opengl_raw::gll::load_with(|s| {
            window.get_proc_address(s)
        });

        window.set_key_polling(true);
        window.set_mouse_button_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_scroll_polling(true);
        window.set_size_polling(true);

        opengl_raw::set_errors(true);

        println!("Preparing Graphical Backend");
        let viewport = Viewport::new(900, 600);

        let settings = ClientSettings::new();
        let mut control_handler = controller::ControlHandler::new(events);
        let movement_action = MovementAction::new(&mut control_handler);
        let world_renderer = WorldRenderer::new(&mut control_handler);


        //glfw.set_swap_interval(SwapInterval::Sync(1));
        println!("Finishing");
        gl::clear_color(0.5f32, 0.6f32, 0.98f32, 1f32);
        gl::viewport(0, 0, 900, 600);


        //gl::enable(gl::CULL_FACE);
        //gl::cull_face(gl::FRONT_FACE);
        //gl::front_face(gl::CW);
        //gl::enable(gl::DEPTH_TEST);

        gl::enable(gl::BLEND);
        gl::blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        // Debug shit
        //gl::enable(gl::DEBUG_OUTPUT);
        //gl::debug_message_callback();

        Self {
            glfw,
            window,
            viewport,
            settings,
            world: None,
            player_id: PlayerId::default(),
            world_renderer,
            control_handler,
            movement_action,
        }
    }
}