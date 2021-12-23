use simdnoise::NoiseBuilder;

use crate::consts;
use crate::consts::TileId;
use crate::world::{Chunk, CHUNK_SIZE};
use crate::world::pos::{ChunkPos, ChunkTilePos};
use crate::world::tile::{Direction, Tile};

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
        let mut chunk = Chunk::new(pos.clone());
        let chunk_x = (pos.x as i32 * CHUNK_SIZE as i32) as f32;
        let chunk_y = (pos.y as u32 * CHUNK_SIZE as u32) as f32;
        //self.generate_surface_terrain(&mut chunk, chunk_x, chunk_y);
        self.generate_caves(&mut chunk, chunk_x, chunk_y);
        //self.generate_surface_terrain(&mut chunk, chunk_x, chunk_y);


        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                chunk.solid_tiles[y][x].variance = ((y as f64 * 2.42378987946378621_f64 * x as f64 * 4.134578901345678013457860_f64) % (u8::MAX as f64)) as u8
            }
        }
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
                    chunk.solid_tiles[y][x] = Tile::id(TileId { id: 1 })
                }
            }
        }
    }

    pub fn generate_caves(&self, chunk: &mut Chunk, chunk_x: f32, chunk_y: f32) {

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                chunk.set(&ChunkTilePos::new(x as u8, y as u8), Tile::id(consts::STONE));
            }
        }
    }
}

pub enum NoiseImpl {}