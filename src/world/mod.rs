mod tile;
mod wall;
mod tick;
mod world_gen;

use std::collections::HashMap;

use crate::registry::{Entity, Fluid, TileId, WallId};
use crate::world::tick::TickHandler;
use crate::world::world_gen::ChunkGenerator;

const CHUNK_SIZE: usize = 24;

pub struct World<'a> {
    chunks: HashMap<ChunkPos, Chunk>,
    tick_handler: TickHandler<'a>,
    chunk_generator: ChunkGenerator,
}

impl<'a> World<'a> {
    pub fn poll_chunk(&mut self, pos: ChunkPos) -> &Chunk {
        let map = &mut self.chunks;
        let option = map.get(&pos);
        option.map_or(
            {
                let chunk = self.chunk_generator.generate_chunk(&pos);
                map.insert(pos, chunk);
                &chunk
            },
            |chunk| {
                chunk
            },
        )
    }
}


// 	8400 × 2400 largest terraria map size
// 	2097120 × 2097120 largest rustaria map size
#[derive(Hash, Eq, PartialEq)]
pub struct ChunkPos {
    x: i16,
    y: u16,
}

pub struct Chunk {
    solid_tiles: [[TileId; CHUNK_SIZE]; CHUNK_SIZE],
    solid_walls: [[WallId; CHUNK_SIZE]; CHUNK_SIZE],
    fluids: Option<[[Fluid; CHUNK_SIZE]; CHUNK_SIZE]>,
    entities: Vec<Entity>,
}
