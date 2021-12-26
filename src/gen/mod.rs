use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::gen::noise::{NoiseGenerator, SUB_BIOME, TERRAIN};
use crate::misc::pos::{ChunkPos, ChunkSubPos};
use crate::misc::util::{CHUNK_SIZE, Direction};
use crate::world::{Chunk, Grid, tile, World};
use crate::world::neighbor::{NeighborAware, NeighborMatrix};
use crate::world::tile::Tile;
use crate::world::wall::Wall;

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

	queue: Vec<ChunkPos>,
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
			queue: Vec::new(),
		}
	}

	pub fn add_chunk(&mut self, pos: &ChunkPos) {
		self.queue.push(pos.clone());
	}

	pub fn generate_chunks(&mut self) -> Option<Vec<(ChunkPos, Chunk)>> {
		if self.queue.is_empty() {
			None
		} else {
			let chunks = self.queue.par_iter().map(|pos| {
				(pos.clone(), self.gen_chunk(pos))
			}).collect();
			self.queue.clear();
			Some(chunks)
		}
	}

	fn gen_chunk(&self, pos: &ChunkPos) -> Chunk {
		let mut chunk = Chunk::new();
		self.generate_terrain(&mut chunk, pos);
		chunk = Self::calc_internal_neighbors::<Wall, Chunk>(chunk);
		chunk = Self::calc_internal_neighbors::<Tile, Chunk>(chunk);
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
					chunk.set(&ChunkSubPos::new(x as u8, y as u8), Tile::id(tile::ASH_BLOCK));
				} else if tile_y < hell_ceiling_height_line as i32 {
					// stuff
				} else if tile_y < cave_height_line as i32 {
					chunk.set(&ChunkSubPos::new(x as u8, y as u8), Tile::id(tile::STONE));
				} else if tile_y < cave_transition_height as i32 {
					chunk.set(&ChunkSubPos::new(x as u8, y as u8), Tile::id(tile::STONE));
				} else if tile_y < terrain_height_line as i32 {
					chunk.set(&ChunkSubPos::new(x as u8, y as u8), Tile::id(tile::DIRT));
				}
			}
		}
	}

	pub fn calc_internal_neighbors<N: NeighborAware, G: Grid<N>>(owner: G) -> G {
		let grid = owner.get_grid();
		// update bottom right neighbors.
		for y in 0..(CHUNK_SIZE - 1) {
			let row = &grid[y];
			let row_below = &grid[y + 1];
			for x in 0..(CHUNK_SIZE - 1) {
				let obj = &row[x];
				let obj_after = &row[x + 1];
				let obj_below = &row_below[x];
				unsafe {
					// mutates the values!!!
					NeighborMatrix::update_neighbor(obj, obj_after, Direction::Right);
					NeighborMatrix::update_neighbor(obj, obj_below, Direction::Top);
				}
			}
		};

		// update right row
		for y in 0..(CHUNK_SIZE - 1) {
			let row = &grid[y];
			let row_below = &grid[y + 1];
			let obj = &row[CHUNK_SIZE - 1];
			let obj_below = &row_below[CHUNK_SIZE - 1];
			unsafe {
				// mutates the values!!!
				NeighborMatrix::update_neighbor(obj, obj_below, Direction::Top);
			}
		};

		// update bottom column
		for x in 0..(CHUNK_SIZE - 1) {
			let row = &grid[CHUNK_SIZE - 1];
			let obj = &row[x];
			let obj_after = &row[x + 1];
			unsafe {
				// mutates the values!!!
				NeighborMatrix::update_neighbor(obj, obj_after, Direction::Right);
			}
		};

		owner
	}
}