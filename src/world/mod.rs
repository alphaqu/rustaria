use std::collections::HashMap;

use crate::Player;
use crate::registry::{Entity, Fluid, TileId, WallId};
use crate::world::tick::TickHandler;
use crate::world::world_gen::ChunkGenerator;

pub mod tile;
pub mod wall;
pub mod tick;
pub mod world_gen;

pub const CHUNK_SIZE: usize = 24;

pub struct World<'a> {
    chunks: HashMap<ChunkPos, Chunk>,
    tick_handler: TickHandler<'a>,
    chunk_generator: ChunkGenerator,
}

impl<'a> World<'a> {
    pub fn new() -> World<'a> {
        Self {
            chunks: HashMap::new(),
            tick_handler: TickHandler::new(),
            chunk_generator: ChunkGenerator::new(69),
        }
    }

    pub fn tick(&mut self) {
        self.tick_handler.tick();
    }

    pub fn poll_chunk(&mut self, pos: &ChunkPos) -> &Chunk {
        return if self.chunks.contains_key(pos) {
            self.chunks.get(pos).unwrap()
        } else {
            let chunk = self.chunk_generator.generate_chunk(pos);
            self.chunks.insert(*pos, chunk);
            self.chunks.get(pos).unwrap()
        };

    }
}


// 	8400 × 2400 largest terraria map size
// 	2097120 × 2097120 largest rustaria map size
#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct ChunkPos {
    pub x: i16,
    pub y: u16,
}

impl ChunkPos {
    pub fn new(x: i16, y: u16) -> ChunkPos {
        Self {
            x,
            y,
        }
    }

    pub fn from_player(player: &Player) -> ChunkPos {
        let chunk_x = (player.pos_x as f32 / CHUNK_SIZE as f32).floor() as i16;
        let chunk_y = (player.pos_y as f32 / CHUNK_SIZE as f32).floor() as u16;
        ChunkPos::new(chunk_x, chunk_y)
    }

}

pub struct Chunk {
    pub solid_tiles: [[TileId; CHUNK_SIZE]; CHUNK_SIZE],
    pub solid_walls: [[WallId; CHUNK_SIZE]; CHUNK_SIZE],
    fluids: Option<[[Fluid; CHUNK_SIZE]; CHUNK_SIZE]>,
    entities: Vec<Entity>,
}

impl Chunk {
    pub fn new() -> Chunk {
        let solid_tiles = [[TileId { id: 0 }; CHUNK_SIZE]; CHUNK_SIZE];
        let solid_walls = [[WallId { id: 0 }; CHUNK_SIZE]; CHUNK_SIZE];

        Self {
            solid_tiles,
            solid_walls,
            fluids: None,
            entities: Vec::new(),
        }
    }

    pub fn set(&mut self, x: usize, y: usize, tile: TileId) {
        self.solid_tiles[y][x] = tile;
    }
}
