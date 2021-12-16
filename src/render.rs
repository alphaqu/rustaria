use std::ffi::CStr;
use std::ptr::null;
use std::sync::mpsc::Receiver;
use std::time::Instant;

use glfw::{Glfw, Window};

use crate::{attach_shader, BufferUsage, Chunk, compile_shader, create_shader, DataType, delete_program, detach_shader, draw_arrays, draw_arrays_instanced, FRAGMENT_SHADER, gl, GLchar, GLenum, gll, GLsizei, GLuint, link_program, Program, read_asset_string, shader_source, Tile, TRIANGLES, Uniform, VERTEX_SHADER, VertexAttribute, VertexBuffer, VertexDivisor, VertexLayout};
use crate::hlgl::{VertexBuilder, Viewport};
use crate::player::{Player, PlayerPos};

pub struct FpsCounter {
    old_fps: Instant,
    frames: u128,
}

impl FpsCounter {
    pub fn new() -> FpsCounter {
        Self { old_fps: Instant::now(), frames: 0 }
    }
    pub fn tick(&mut self) {
        self.frames = self.frames + 1;
        if self.old_fps.elapsed().as_millis() > 1000 {
            println!("FPS: {}", self.frames);
            self.frames = 0;
            self.old_fps = Instant::now();
        }
    }
}


pub struct Renderer {
    glfw: Glfw,
    window: Window,
    event: Receiver<()>,
}


impl Renderer {}

pub struct Buffer {}

pub struct BakedTile {}

pub struct BakedChunk {
    x: f32,
    y: f32,
    layout: VertexLayout,
}

impl BakedChunk {
    pub fn new(chunk: &Chunk, player: &Player, viewport: &Viewport, scale: u32) -> BakedChunk {
        Self {
            x: (chunk.chunk_x as i64 * 8) as f32,
            y: (chunk.chunk_y as u32 * 8) as f32,
            layout: Self::build(chunk, player, viewport, scale),
        }
    }
    pub fn build(chunk: &Chunk, player: &Player, viewport: &Viewport, scale: u32) -> VertexLayout {
        let mut pos_vb = VertexBuilder::new(viewport);

        let mut color_vb = VertexBuilder::new(viewport);

        for y in 0..8 {
            let tiles_y = &chunk.tile_data[y];
            let tile_y = (-(chunk.chunk_y as i32 * 8i32) + y as i32) as f32;
            let screen_y = tile_y - player.pos.y;

            for x in 0..8 {
                let tile = &tiles_y[x];
                if tile.id.id == 0 {
                    continue;
                }
                let tile_x = (-(chunk.chunk_x as i32 * 8i32) + x as i32) as f32;
                let screen_x = tile_x - player.pos.x;

                let color = tile.id.id as f32;
                color_vb.add(color);
                color_vb.add(color);
                color_vb.add(color);
                color_vb.add(color);
                color_vb.add(color);
                color_vb.add(color);
                pos_vb.quad(screen_x * scale as f32, screen_y * scale as f32, scale as f32, scale as f32);
            }
        }


        let mut layout = VertexLayout::new(2);
        layout.add_vbo(0, color_vb.export(), 1, BufferUsage::StaticDraw, DataType::Float, VertexDivisor::Vertex);
        layout.add_vbo(1, pos_vb.export(), 2, BufferUsage::StaticDraw, DataType::Float, VertexDivisor::Vertex);
        layout
    }

    pub fn draw(&self) {
        self.layout.bind();
        for i in 0..(8 * 8) {
            draw_arrays(TRIANGLES, ((6 * i) as i32), 6);
        }
        self.layout.unbind();
    }
}

pub struct PlayerRenderer {
    program: Program,
    layout: VertexLayout,
}

impl PlayerRenderer {
    pub fn new(viewport: &Viewport, scale: u32) -> PlayerRenderer {
        let program = Program::new(
            read_asset_string("shader/quad-fragment.glsl"),
            read_asset_string("shader/quad-vertex.glsl"),
        );

        let mut pos_vb = VertexBuilder::new(viewport);
        let width = 2f32;
        let height = 3f32;
        let x = (viewport.get_width() as f32 / 2f32) + (width / 2f32);
        let y = (viewport.get_height() as f32 / 2f32) + (height / 2f32);
        pos_vb.quad(x * scale as f32, y * scale as f32, width * scale as f32, height * scale as f32);

        pos_vb.pos_x(3f32);
        let mut layout = VertexLayout::new(1);
        layout.add_vbo(0, pos_vb.export(), 2, BufferUsage::StaticDraw, DataType::Float, VertexDivisor::Vertex);

        Self {
            program,
            layout,
        }
    }

    pub fn draw(&self) {
        self.program.bind();
        self.layout.bind();
        draw_arrays(TRIANGLES, 0, 6);
        self.layout.unbind();
        self.program.unbind();
    }
}

pub struct TileRenderer {
    pub program: Program,
    pub player_pos: Uniform,
    pub chunks: Vec<BakedChunk>,
}

impl TileRenderer {
    pub fn new() -> TileRenderer {
        let program = Program::new(
            read_asset_string("shader/tile-fragment.glsl"),
            read_asset_string("shader/tile-vertex.glsl"),
        );
        Self {
            player_pos: Uniform::new(&program, "playerPos"),
            program,
            chunks: Vec::new(),
        }
    }

    pub fn add_chunk(&mut self, chunk: &Chunk, player: &Player, viewport: &Viewport, tile_size: u32) {
        self.chunks.push(BakedChunk::new(chunk, player, viewport, tile_size));
    }

    pub fn set_tile_viewport(&mut self, viewport: &Viewport, tile_size: u32) {}


    pub fn draw(&self, player: &Player) {
        self.program.bind();
        self.player_pos.uniform_2f(player.pos.x, player.pos.y);
        for x in &self.chunks {
            x.draw();
        }
        self.program.unbind();
    }
}