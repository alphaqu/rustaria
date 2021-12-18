use std::collections::HashMap;
use std::fs::read_dir;
use std::ops::Deref;
use std::path::Path;

use image::GenericImageView;

use crate::{Player, read_asset_string, Settings};
use crate::client::opengl::gl;
use crate::client::opengl::gl::{BufferUsage, DataType, VertexDivisor};
use crate::client::opengl::hlgl::{Atlas, AtlasGroup, AtlasImage, Image, Program, Uniform, VertexLayout};
use crate::client::viewport::Viewport;
use crate::registry::TileId;
use crate::world::{Chunk, CHUNK_SIZE, ChunkPos, World};

pub struct ChunkRenderer {
    program: Program,
    player_pos: Uniform,
    gl_zoom: Uniform,

    texture_sampler: Uniform,
    tile_atlas: Atlas,

    old_player_pos: ChunkPos,
    chunks: HashMap<ChunkPos, BakedChunk>,
}

impl ChunkRenderer {
    pub fn new(player: &Player) -> ChunkRenderer {
        let program = Program::new(
            read_asset_string("shader/tile-fragment.glsl"),
            read_asset_string("shader/tile-vertex.glsl"),
        );

        let path = Path::new("C:\\Program Files (x86)\\inkscape\\cppProjects\\rustaria\\assets\\sprite\\tile");

        println!("Reading images");
        let mut images = HashMap::new();
        for entry in read_dir(path).unwrap() {
            let dir = entry.unwrap();
            let buf = dir.path();
            let result = image::open(buf.clone()).unwrap();
            let names: Vec<&str> = buf.to_str().unwrap().split('_').collect();
            let names2: Vec<&str> = names.get(1).unwrap().split('.').collect();
            let id: u32 = names2.get(0).unwrap().parse().unwrap();

            images.insert(id, Image {
                width: result.width(),
                height: result.height(),
                data: Vec::from(result.as_bytes()),
            });
        }


        Self {
            player_pos: Uniform::new(&program, "glPlayerPos"),
            gl_zoom: Uniform::new(&program, "glZoom"),
            texture_sampler: Uniform::new(&program, "texture_sampler"),
            tile_atlas: Atlas::create(images, 1024, 1024, AtlasGroup::Tiles),
            old_player_pos: ChunkPos::from_player(player),
            program,
            chunks: HashMap::new(),
        }
    }

    pub fn tick(&mut self, viewport: &Viewport, player: &Player, world: &mut World, settings: &Settings) {
        let pos = ChunkPos::from_player(player);

        self.process_chunks(viewport, world, &pos, settings.render_distance);
    }

    pub fn rebuild(&mut self, viewport: &Viewport, player: &Player, world: &mut World, settings: &Settings) {
        self.chunks.clear();
        self.tick(viewport, player, world, settings);
    }

    pub fn draw(&self, viewport: &Viewport, player: &Player, settings: &Settings) {
        self.program.bind();
        self.player_pos.uniform_2f(player.pos_x * viewport.gl_tile_width, player.pos_y * viewport.gl_tile_height);
        self.gl_zoom.uniform_1f(settings.zoom);
        self.texture_sampler.uniform_1i(0);
        self.tile_atlas.bind();

        if settings.cull_chunks {
            let player_chunk = ChunkPos::from_player(player);
            let chunks_x = (1f32 / (viewport.gl_tile_width * CHUNK_SIZE as f32)) * settings.zoom;
            let chunks_y = (1f32 / (viewport.gl_tile_height * CHUNK_SIZE as f32)) * settings.zoom;

            for (pos, baked_chunk) in &self.chunks {
                if ((pos.x as f32 - player_chunk.x as f32).abs() > (chunks_x + 1f32)) || ((pos.y as f32 - player_chunk.y as f32).abs() > (chunks_y + 1f32)) {
                    continue;
                }

                baked_chunk.draw();
            }
        } else {
            for (pos, baked_chunk) in &self.chunks {
                baked_chunk.draw();
            }
        }
        self.program.unbind();
    }

    pub fn process_chunks(&mut self, viewport: &Viewport, world: &mut World, player_pos: &ChunkPos, render_distance: u16) {
        let render_distance_half = (render_distance as i32 / 2i32) as i32;

        let mut lookup_pos = player_pos.clone();
        for y in 0..(render_distance + 1) {
            let chunk_y: i32 = player_pos.y as i32 + (y as i32 - render_distance_half);
            if chunk_y < 0 {
                continue;
            }
            lookup_pos.y = chunk_y as u16;

            for x in 0..(render_distance + 1) {
                lookup_pos.x = (player_pos.x as i32 + (x as i32 - render_distance_half)) as i16;
                if !self.chunks.contains_key(&lookup_pos) {
                    self.chunks.insert(lookup_pos.clone(), BakedChunk::new(viewport, world, &lookup_pos, &self.tile_atlas));
                }
            }
        }

        self.chunks.retain(|pos, baked_chunk| {
            ((player_pos.y as i32 - pos.y as i32).abs() < (render_distance as i32) && (player_pos.x as i32 - pos.x as i32).abs() < (render_distance as i32))
        })
    }
}

pub struct BakedChunk {
    layout: VertexLayout,
    vertices: u32,
}

impl BakedChunk {
    pub fn new(viewport: &Viewport, world: &mut World, pos: &ChunkPos, tile_atlas: &Atlas) -> BakedChunk {
        let mut builder = ChunkVertexBuilder::new(viewport, pos);

        let mut vertices = 0u32;
        let chunk = world.poll_chunk(pos);
        for y in 0..CHUNK_SIZE {
            let tiles_y = &chunk.solid_tiles[y];
            for x in 0..CHUNK_SIZE {
                let tile_x = &tiles_y[x];

                if tile_x.id == 0 {
                    continue;
                }

                vertices = vertices + 6; // quad
                builder.add_tile(x, y, tile_atlas.get_image(TileId { id: tile_x.id }));
            }
        }

        Self {
            layout: builder.export(),
            vertices,
        }
    }

    pub fn draw(&self) {
        self.layout.bind();
        gl::draw_arrays(gl::TRIANGLES, 0, self.vertices as i32);
        self.layout.unbind();
    }
}


pub enum TileImageType {
    Full,
    Standalone,
    StraightVertical,
    StraightHorizontal,
    TopFlat,
    TopCap,
    TopReach,
    TopLeftCorner,
    DownFlat,
    DownCap,
    DownReach,
    DownLeftCorner,
    LeftFlat,
    LeftCap,
    LeftReach,
    TopRightCorner,
    RightFlat,
    RightCap,
    RightReach,
    DownRightCorner,
}

impl TileImageType {
    pub fn get_pos(&self) -> (u32, u32) {
        match self {
            TileImageType::Full() => (0, 0),
            TileImageType::Standalone() => (1, 0),
            TileImageType::StraightVertical() => (2, 0),
            TileImageType::StraightHorizontal() => (3, 0),
            TileImageType::TopFlat() => (0, 1),
            TileImageType::TopCap() => (1, 1),
            TileImageType::TopReach() => (2, 1),
            TileImageType::TopLeftCorner() => (3, 1),
            TileImageType::DownFlat() => (0, 2),
            TileImageType::DownCap() => (1, 2),
            TileImageType::DownReach() => (2, 2),
            TileImageType::DownLeftCorner() => (3, 2),
            TileImageType::LeftFlat() => (0, 3),
            TileImageType::LeftCap() => (1, 3),
            TileImageType::LeftReach() => (2, 3),
            TileImageType::TopRightCorner() => (3, 3),
            TileImageType::RightFlat() => (0, 4),
            TileImageType::RightCap() => (1, 4),
            TileImageType::RightReach() => (2, 4),
            TileImageType::DownRightCorner() => (3, 4),
        }
    }
}

pub struct ChunkVertexBuilder {
    pos: Vec<f32>,
    textures: Vec<f32>,
    gl_chunk_x: f32,
    gl_chunk_y: f32,
    gl_tile_width: f32,
    gl_tile_height: f32,
}

impl ChunkVertexBuilder {
    pub fn new(viewport: &Viewport, chunk_pos: &ChunkPos) -> ChunkVertexBuilder {
        Self {
            pos: Vec::new(),
            textures: Vec::new(),
            gl_chunk_x: (chunk_pos.x as f32) * (viewport.gl_tile_width * CHUNK_SIZE as f32),
            gl_chunk_y: (chunk_pos.y as f32) * (viewport.gl_tile_height * CHUNK_SIZE as f32),
            gl_tile_width: viewport.gl_tile_width,
            gl_tile_height: viewport.gl_tile_height,
        }
    }

    pub fn add_tile(&mut self, x: usize, y: usize, image: &AtlasImage) {
        let gl_x = self.gl_chunk_x + (x as f32 * self.gl_tile_width);
        let gl_y = self.gl_chunk_y + (y as f32 * self.gl_tile_height);
        self.add_internal(gl_x, gl_y, self.gl_tile_width, self.gl_tile_height, image);
    }

    pub fn export(self) -> VertexLayout {
        let mut layout = VertexLayout::new(2);
        layout.add_vbo(0, self.pos, 2, BufferUsage::StaticDraw, DataType::Float, VertexDivisor::Vertex);
        layout.add_vbo(1, self.textures, 2, BufferUsage::StaticDraw, DataType::Float, VertexDivisor::Vertex);
        layout
    }

    fn add_internal(&mut self, x: f32, y: f32, width: f32, height: f32, image: &AtlasImage) {
        let x1 = (image.width / 12f32);
        let x2 = (image.height / 5f32);

        self.pos.push(x);
        self.textures.push(image.x);
        self.pos.push(y + height);
        self.textures.push(image.y + x2);

        self.pos.push(x);
        self.textures.push(image.x);
        self.pos.push(y);
        self.textures.push(image.y);

        self.pos.push(x + width);
        self.textures.push(image.x + x1);
        self.pos.push(y + height);
        self.textures.push(image.y + x2);

        self.pos.push(x + width);
        self.textures.push(image.x + x1);
        self.pos.push(y + height);
        self.textures.push(image.y + x2);

        self.pos.push(x);
        self.textures.push(image.x);
        self.pos.push(y);
        self.textures.push(image.y);

        self.pos.push(x + width);
        self.textures.push(image.x + x1);
        self.pos.push(y);
        self.textures.push(image.y);
    }
}
