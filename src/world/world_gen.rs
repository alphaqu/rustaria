use simdnoise::NoiseBuilder;

use crate::registry::TileId;
use crate::world::{Chunk, CHUNK_SIZE, ChunkPos};

pub struct WorldGenerationOptions {
    surface_y: u32,
    surface_height: u32,
    cave_chance: f32,
}

pub struct ChunkGenerator {
    options: WorldGenerationOptions,
    seed: i32,
}

impl ChunkGenerator {
    pub fn new(seed: i32) -> ChunkGenerator {
        Self {
            options: WorldGenerationOptions {
                surface_y: 100,
                surface_height: 40,
                cave_chance: 0.5f32,
            },
            seed,
        }
    }

    pub fn generate_chunk(&self, pos: &ChunkPos) -> Chunk {
        let mut chunk = Chunk::new();
        let chunk_x = (pos.x as i32 * CHUNK_SIZE as i32) as f32;
        let chunk_y = (pos.y as u32 * CHUNK_SIZE as u32) as f32;
        //self.generate_surface_terrain(&mut chunk, chunk_x, chunk_y);
        self.generate_caves(&mut chunk, chunk_x, chunk_y);
        chunk
    }

    pub fn generate_surface_terrain(&self, chunk: &mut Chunk, chunk_x: f32, chunk_y: f32) {
        let noise = NoiseBuilder::fbm_1d_offset(chunk_x, CHUNK_SIZE)
            .with_octaves(4)
            .with_seed(self.seed)
            .with_freq(0.04)
            .with_gain(1f32)
            .generate_scaled(0f32, self.options.surface_height as f32);
        for x in 0..CHUNK_SIZE {
            let surface_height: f32 = noise[x];
            for y in 0..CHUNK_SIZE {
                if (y as f32 + chunk_y) < surface_height + self.options.surface_y as f32 {
                    chunk.set(x, y, TileId { id: 1 })
                }
            }
        }
    }

    pub fn generate_caves(&self, chunk: &mut Chunk, chunk_x: f32, chunk_y: f32) {
        for y in 0..CHUNK_SIZE {
            if y > 12 {
                let options = [
                    1, 2, 6, 7, 8, 9, 22, 23, 25, 30, 32, 37
                ];
                for x in 0..CHUNK_SIZE {
                    chunk.set(x, y, TileId { id: options[x % options.len()] })
                }
            }
        }
    }
}

pub enum NoiseImpl {}