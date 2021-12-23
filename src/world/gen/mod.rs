use crate::{consts};
use crate::util::{CHUNK_SIZE, Direction};
use crate::world::{Chunk, Grid, World};
use crate::world::gen::noise::{NoiseGenerator, SUB_BIOME, TERRAIN};
use crate::pos::{ChunkPos, ChunkSubPos};
use crate::world::tile::Tile;

mod feature;
mod gen_const;
mod noise;

pub struct WorldGenerator {
	noise: NoiseGenerator,
	terrain_height: u32,
	cave_transition_height: u32,
	cave_height: u32,
	hell_transition_height: u32,
	hell_lava: u32,
}

impl WorldGenerator {
	pub fn new(seed: u64) -> WorldGenerator {
		Self {
			noise: NoiseGenerator::new(seed),
			terrain_height: 50,
			cave_transition_height: 50,
			cave_height: 600,
			hell_transition_height: 50,
			hell_lava: 150,
		}
	}

	pub fn gen_chunk(&self, pos: &ChunkPos) -> Chunk {
		let mut chunk = Chunk::new(*pos);
		self.generate_terrain(&mut chunk, pos);
		chunk
	}
	pub fn generate_terrain(&self, chunk: &mut Chunk, pos: &ChunkPos) {
		for x in 0..CHUNK_SIZE {
			let tile_x = (x as i32 + (pos.x as i32 * CHUNK_SIZE as i32)) as i32;
			let hell_floor_height_line =
				self.noise.get_2d_range(tile_x, self.hell_lava, self.hell_lava, TERRAIN);
			let hell_ceiling_height_line =
				self.hell_lava as f64 +
					self.noise.get_2d_range(tile_x, self.hell_transition_height, self.hell_transition_height, TERRAIN);
			let cave_height_line =
				self.hell_lava as f64 +
					self.hell_transition_height as f64 +
					self.noise.get_2d_range(tile_x, self.cave_height, self.cave_height, TERRAIN);

			let cave_transition_height =
				self.hell_lava as f64 +
					self.hell_transition_height as f64 +
					self.cave_height as f64 +
					self.noise.get_2d_range(tile_x, self.cave_transition_height, self.cave_transition_height, TERRAIN);
			let terrain_height_line =
				self.hell_lava as f64 + self.hell_transition_height as f64 + self.cave_transition_height as f64 + self.cave_height as f64 +
					self.noise.get_2d_range(tile_x, self.terrain_height, self.terrain_height, TERRAIN);

			for y in 0..CHUNK_SIZE {
				let tile_y = (y as i32 + (pos.y as i32 * CHUNK_SIZE as i32)) as i32;
				if tile_y < hell_floor_height_line as i32 {
					chunk.set(&ChunkSubPos::new(x as u8, y as u8), Tile::id(consts::TILE_ASH_BLOCK));
				} else if tile_y < hell_ceiling_height_line as i32 {
					// stuff
				} else if tile_y < cave_height_line as i32 {
					chunk.set(&ChunkSubPos::new(x as u8, y as u8), Tile::id(consts::TILE_STONE));
				} else if tile_y < cave_transition_height as i32 {
					chunk.set(&ChunkSubPos::new(x as u8, y as u8), Tile::id(consts::TILE_STONE));
				} else if tile_y < terrain_height_line as i32 {
					chunk.set(&ChunkSubPos::new(x as u8, y as u8), Tile::id(consts::TILE_DIRT));
				}
			}
		}
	}

	pub fn calc_neighbors(&self, chunk: Chunk, pos: &ChunkPos, world: &mut World) -> Chunk {
		chunk
	}
}