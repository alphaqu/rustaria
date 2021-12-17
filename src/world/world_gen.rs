use crate::world::world::{Chunk, ChunkPos};

pub struct ChunkGenerator {}

impl ChunkGenerator {
    pub fn generate_chunk(&self, pos: &ChunkPos) -> Chunk {
        todo!("Generate Chunks")
    }
}

pub enum NoiseImpl {}