use std::collections::HashMap;
use std::fs::read_dir;
use std::path::Path;

use glfw::Key;

use crate::{Player, read_asset_string};
use crate::client::client_settings::ClientSettings;
use crate::client::controller::{ControlHandler, Event, EventKey, EventType, KeyMapping};
use crate::client::opengl::gl;
use crate::client::opengl::hlgl::{Atlas, AtlasSettings, Image, Sampler2d};
use crate::client::opengl::sgl::{Program, Uniform, UniformType};
use crate::client::render::baked_chunk::BakedChunk;
use crate::client::viewport::Viewport;
use crate::misc::pos::{ChunkPos, WorldPos};
use crate::misc::util::{CHUNK_SIZE, Direction};
use crate::world::neighbor::NeighborAware;
use crate::world::World;

pub struct WorldRenderer {
    program: Program,

    player_pos: Uniform<(f32, f32)>,
    gl_zoom: Uniform<f32>,
    texture_sampler: Uniform<Sampler2d>,
    atlas: Atlas,

    baked_chunks: HashMap<ChunkPos, BakedChunk>,

    debug_mode: bool,
    cull_chunks: bool,
    debug_mode_key: EventKey,
    cull_chunks_key: EventKey,
    rebuild_chunks_key: EventKey,
}

impl WorldRenderer {
    pub fn new(control_handler: &mut ControlHandler) -> WorldRenderer {
        let program = Program::create(
            read_asset_string("shader/tile-vertex.glsl"),
            read_asset_string("shader/tile-fragment.glsl"),
            None,
        );


        println!("Reading images");
        let mut images = Vec::new();
        for entry in read_dir(Path::new("./assets/sprite/tile")).unwrap() {
            let dir = entry.unwrap();
            let buf = dir.path();

            let image = Image::load(buf.as_ref());
            images.push(image);
        }

        for entry in read_dir(Path::new("./assets/sprite/wall")).unwrap() {
            let dir = entry.unwrap();
            let buf = dir.path();

            let image = Image::load(buf.as_ref());
            images.push(image);
        }

        let atlas = Atlas::new(images, AtlasSettings {
            mipmaps: 3
        });


        Self {
            player_pos: program.get_uniform("player_pos"),
            gl_zoom: program.get_uniform("zoom"),
            texture_sampler: program.get_uniform("texture_sampler"),
            atlas,
            program,
            baked_chunks: HashMap::new(),
            debug_mode: false,
            cull_chunks: true,
            debug_mode_key: control_handler.register(Event::new(EventType::new_toggle(false), KeyMapping::key(Key::F1), "debug.mode")),
            cull_chunks_key: control_handler.register(Event::new(EventType::new_toggle(true), KeyMapping::key(Key::F2), "debug.cull_chunks")),
            rebuild_chunks_key: control_handler.register(Event::new(EventType::new_request(), KeyMapping::key(Key::F3), "debug.rebuild_chunks")),
        }
    }

    pub fn event_apply(&mut self, control_handler: &ControlHandler) {
        if let EventType::Toggle { state } = control_handler.acquire(&self.debug_mode_key) {
            self.debug_mode = *state;
        }

        if let EventType::Toggle { state } = control_handler.acquire(&self.cull_chunks_key) {
            self.cull_chunks = *state;
        }

        if let EventType::Request { requests } = control_handler.acquire(&self.rebuild_chunks_key) {
            if *requests > 0 {
                self.rebuild_all();
                println!("Rebuilt chunks");
            }
        }
    }

    pub fn tick(
        &mut self,
        world: &World,
        viewport: &Viewport,
        player: &Player,
        settings: &ClientSettings,
    ) {
        let player_pos = ChunkPos::from_player(player);
        let render_distance = settings.render_distance as i32;

        let mut lookup_pos = player_pos;
        for y in -render_distance..render_distance {
            let chunk_y: i32 = player_pos.y as i32 + y;
            if chunk_y < 0 {
                continue;
            }
            lookup_pos.y = chunk_y as u16;

            for x in -render_distance..render_distance {
                lookup_pos.x = (player_pos.x as i32 + x) as i16;
                if !self.baked_chunks.contains_key(&lookup_pos) {
                    let pos1 = &lookup_pos;
                    if let Some(baked_chunk) = BakedChunk::new(&self.program, viewport, world, pos1, &self.atlas) {
                        self.baked_chunks.insert(lookup_pos, baked_chunk);
                    }
                }
            }
        }
    }

    pub fn tile_change(&mut self, pos: &WorldPos) {
        let chunk_tile_pos = pos.get_chunk_sub_pos();
        for dir in Direction::iter() {
            if chunk_tile_pos.is_border(dir) {
                if let Some(pos) = pos.get_chunk_pos().shift(dir) { self.baked_chunks.remove(&pos); }
            }
        }
        self.baked_chunks.remove(pos.get_chunk_pos());
    }

    pub fn draw(&self, viewport: &Viewport, player: &Player, settings: &ClientSettings) {
        self.program.bind();
        self.player_pos.apply((player.pos_x * viewport.gl_tile_width, player.pos_y * viewport.gl_tile_height));
        self.gl_zoom.apply(settings.zoom);
        self.texture_sampler.apply(Sampler2d::new(0));

        if self.cull_chunks {
            let player_chunk = ChunkPos::from_player(player);
            let chunks_x = (1f32 / (viewport.gl_tile_width * CHUNK_SIZE as f32)) * settings.zoom;
            let chunks_y = (1f32 / (viewport.gl_tile_height * CHUNK_SIZE as f32)) * settings.zoom;

            for (pos, baked_chunk) in &self.baked_chunks {
                if ((pos.x as f32 - player_chunk.x as f32).abs() > (chunks_x + 1f32))
                    || ((pos.y as f32 - player_chunk.y as f32).abs() > (chunks_y + 1f32))
                {
                    continue;
                }

                baked_chunk.draw(gl::TRIANGLES);
            }
        } else {
            for baked_chunk in self.baked_chunks.values() {
                baked_chunk.draw(gl::TRIANGLES);
            }
        }
        self.program.unbind();
    }

    pub fn rebuild_all(&mut self) {
        self.baked_chunks.clear();
    }

    pub fn rebuild_chunk(&mut self, chunk_pos: &ChunkPos) {
        self.baked_chunks.remove(chunk_pos);
    }
}

pub enum NeighborImageLocation {
    Full,
    Standalone,
    StraightVertical,
    StraightHorizontal,
    TopFlat,
    TopCap,
    TopLeftCorner,
    DownFlat,
    DownCap,
    DownLeftCorner,
    LeftFlat,
    LeftCap,
    TopRightCorner,
    RightFlat,
    RightCap,
    DownRightCorner,
}

impl NeighborImageLocation {
    pub fn from<N: NeighborAware>(object: &N) -> NeighborImageLocation {
        use crate::world::neighbor::NeighborType::{Air, Same};
        let matrix = object.get_neighbor_matrix();
        match (
            matrix.get_neighbor_type(Direction::Top),
            matrix.get_neighbor_type(Direction::Down),
            matrix.get_neighbor_type(Direction::Left),
            matrix.get_neighbor_type(Direction::Right),
        ) {
            (Same, Same, Same, Same) => NeighborImageLocation::Full,
            (Air, Air, Air, Air) => NeighborImageLocation::Standalone,
            (Same, Same, Air, Air) => NeighborImageLocation::StraightVertical,
            (Air, Air, Same, Same) => NeighborImageLocation::StraightHorizontal,

            (Air, Same, Same, Same) => NeighborImageLocation::TopFlat,
            (Air, Same, Air, Air) => NeighborImageLocation::TopCap,
            (Air, Same, Air, Same) => NeighborImageLocation::TopLeftCorner,
            (Air, Same, Same, Air) => NeighborImageLocation::TopRightCorner,

            (Same, Air, Same, Same) => NeighborImageLocation::DownFlat,
            (Same, Air, Air, Air) => NeighborImageLocation::DownCap,
            (Same, Air, Air, Same) => NeighborImageLocation::DownLeftCorner,
            (Same, Air, Same, Air) => NeighborImageLocation::DownRightCorner,

            (Same, Same, Air, Same) => NeighborImageLocation::LeftFlat,
            (Air, Air, Air, Same) => NeighborImageLocation::LeftCap,
            (Same, Same, Same, Air) => NeighborImageLocation::RightFlat,
            (Air, Air, Same, Air) => NeighborImageLocation::RightCap,
            _ => NeighborImageLocation::Full,
        }
    }

    pub fn get_tile_pos(&self) -> (u32, u32) {
        match self {
            NeighborImageLocation::Full => (0, 0),
            NeighborImageLocation::Standalone => (1, 0),
            NeighborImageLocation::StraightVertical => (2, 0),
            NeighborImageLocation::StraightHorizontal => (3, 0),
            NeighborImageLocation::TopFlat => (0, 1),
            NeighborImageLocation::TopCap => (1, 1),
            //NeighborImageLocation::TopReach => (2, 1),
            NeighborImageLocation::TopLeftCorner => (3, 1),
            NeighborImageLocation::DownFlat => (0, 2),
            NeighborImageLocation::DownCap => (1, 2),
            //NeighborImageLocation::DownReach => (2, 2),
            NeighborImageLocation::DownLeftCorner => (3, 2),
            NeighborImageLocation::LeftFlat => (0, 3),
            NeighborImageLocation::LeftCap => (1, 3),
            //NeighborImageLocation::LeftReach => (2, 3),
            NeighborImageLocation::TopRightCorner => (3, 3),
            NeighborImageLocation::RightFlat => (0, 4),
            NeighborImageLocation::RightCap => (1, 4),
            //NeighborImageLocation::RightReach => (2, 4),
            NeighborImageLocation::DownRightCorner => (3, 4),
        }
    }
    pub fn get_wall_pos(&self) -> ((f32, f32), (f32, f32)) {
        match self {
            NeighborImageLocation::Full => ((0.5, 0.5), (1.0, 1.0)),
            NeighborImageLocation::Standalone => ((0.0, 0.0), (2.0, 2.0)),
            NeighborImageLocation::StraightVertical => ((0.0, 0.5), (2.0, 1.0)),
            NeighborImageLocation::StraightHorizontal => ((0.5, 0.0), (1.0, 2.0)),

            NeighborImageLocation::TopFlat => ((0.5, 0.0), (1.0, 1.5)),
            NeighborImageLocation::TopCap => ((0.0, 0.0), (2.0, 1.5)),
            NeighborImageLocation::TopLeftCorner => ((0.0, 0.0), (1.5, 1.5)),

            NeighborImageLocation::DownFlat => ((0.5, 0.5), (1.0, 1.5)),
            NeighborImageLocation::DownCap => ((0.0, 0.5), (2.0, 1.5)),
            NeighborImageLocation::DownLeftCorner => ((0.0, 0.5), (1.5, 1.5)),

            NeighborImageLocation::LeftFlat => ((0.0, 0.5), (1.5, 1.0)),
            NeighborImageLocation::LeftCap => ((0.0, 0.0), (1.5, 2.0)),
            NeighborImageLocation::TopRightCorner => ((0.5, 0.0), (1.5, 1.5)),

            NeighborImageLocation::RightFlat => ((0.5, 0.5), (1.5, 1.0)),
            NeighborImageLocation::RightCap => ((0.5, 0.0), (1.5, 2.0)),
            NeighborImageLocation::DownRightCorner => ((0.5, 0.5), (1.5, 1.5)),
        }
    }
}

