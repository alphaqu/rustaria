extern crate glfw;

use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::Duration;

use glfw::{Action, Context, Key, SwapInterval, WindowEvent};

use gl::*;
use gll::types::*;

use crate::gll::{COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT, VERTEX_SHADER};
use crate::opengl::gl;
use crate::opengl::gl::clear_color;
use crate::opengl::hlgl::Viewport;
use crate::opengl::render::{FpsCounter, PlayerRenderer, TileRenderer};
use crate::player::{Controller, Player, PlayerPos};
use crate::registry::{Chunk, Identifier, Tile};

mod registry;
mod opengl;
mod player;
mod tile_render;

mod gll {
    include!("C:\\Program Files (x86)\\inkscape\\gl-rs-bindings\\bindings.rs");
}

fn main() {
    println!("Generating World");

    let i = Tile { id: Identifier { id: 0 } };
    let d = Tile { id: Identifier { id: 1 } };

    let chunk2 = Chunk::parse_debug(4, 1,
                                    [
                                        [0, 0, 0, 0, 0, 0, 0, 0],
                                        [0, 0, 0, 1, 1, 0, 0, 0],
                                        [0, 0, 1, 1, 1, 1, 0, 0],
                                        [0, 0, 0, 1, 1, 0, 0, 0],
                                        [0, 0, 0, 1, 1, 0, 0, 0],
                                        [0, 0, 0, 1, 1, 0, 0, 0],
                                        [0, 0, 0, 1, 1, 0, 0, 0],
                                        [0, 0, 0, 1, 1, 0, 0, 0],
                                    ]);
    let chunk = Chunk::parse_debug(4, 0,
                                   [
                                       [0, 0, 0, 1, 1, 0, 0, 0],
                                       [0, 0, 0, 1, 1, 0, 0, 0],
                                       [0, 0, 0, 1, 1, 0, 0, 0],
                                       [0, 0, 0, 1, 1, 0, 0, 0],
                                       [0, 1, 1, 1, 1, 1, 1, 0],
                                       [0, 1, 1, 1, 1, 1, 1, 0],
                                       [0, 1, 1, 1, 1, 1, 1, 0],
                                       [0, 1, 1, 1, 1, 1, 1, 0],
                                   ]);


    println!("Launching GLFW");
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    println!("Launching Window");
    let (mut window, events) = glfw
        .create_window(900, 600, "Hello this is window", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.set_size_polling(true);
    window.make_current();


    println!("Setting GL Context");
    gll::load_with(|s| glfw.get_proc_address_raw(s));
    gll::Viewport::load_with(|s| glfw.get_proc_address_raw(s));

    println!("Preparing Graphical Backend");

    glfw.set_swap_interval(SwapInterval::Sync(1));
    let mut viewport = Viewport::new(900, 600);
    let mut tile_renderer = TileRenderer::new();
    let mut player_renderer = PlayerRenderer::new(&viewport, 8);


    let mut fps_counter = FpsCounter::new();

    println!("Finishing");
    gl::clear_color(0.5f32, 0.6f32, 0.98f32, 1f32);
    gl::viewport(0, 0, 900, 600);

    let mut player = Player {
        pos: PlayerPos { x: 0.0, y: 0.0 },
        speed: 11.36f32,
        velocity_x: 0.0,
        velocity_y: 0.0,
        controller: Controller {
            w: false,
            a: false,
            s: false,
            d: false,
        },
    };


    tile_renderer.add_chunk(&chunk, &player, &viewport, 8);
    tile_renderer.add_chunk(&chunk2, &player, &viewport, 8);

    while !window.should_close() {
        fps_counter.tick();

        clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);

        tile_renderer.draw(&player);
        player_renderer.draw();

        window.swap_buffers();

        glfw.poll_events();

        let mut resize = false;
        let mut resize_width = 0;
        let mut resize_height = 0;

        let mut controller = &mut player.controller;
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                WindowEvent::Key(Key::W, _, Action::Press, _) => controller.w = true,
                WindowEvent::Key(Key::W, _, Action::Release, _) => controller.w = false,
                WindowEvent::Key(Key::A, _, Action::Press, _) => controller.a = true,
                WindowEvent::Key(Key::A, _, Action::Release, _) => controller.a = false,
                WindowEvent::Key(Key::S, _, Action::Press, _) => controller.s = true,
                WindowEvent::Key(Key::S, _, Action::Release, _) => controller.s = false,
                WindowEvent::Key(Key::D, _, Action::Press, _) => controller.d = true,
                WindowEvent::Key(Key::D, _, Action::Release, _) => controller.d = false,
                WindowEvent::Size(width, height) => {
                    resize = true;
                    resize_width = width;
                    resize_height = height;
                }
                _ => {}
            }
        }

        player.tick();
        if resize {
            viewport.resize(resize_width, resize_height);
            tile_renderer.set_tile_viewport(&viewport, 24);
        }
    }
}


fn read_asset_string(path: &str) -> String {
    let mut file = File::open("./assets/".to_owned() + path).unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string);
    string
}

