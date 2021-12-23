
use std::collections::{HashMap, HashSet};

use neighbor::NeighborAware;

use crate::consts::{Entity};

use crate::pos::{ChunkPos, ChunkSubPos, WorldPos};
use crate::util::{CHUNK_SIZE, Direction};
use crate::world::gen::WorldGenerator;
use crate::world::neighbor::{NeighborMatrix};
use crate::world::tick::TickHandler;
use crate::world::tile::Tile;
use crate::world::wall::Wall;

pub mod tile;
pub mod wall;
pub mod tick;
pub mod neighbor;
mod gen;

pub struct World<'a> {
	pub chunk_updates: HashSet<ChunkPos>,
	chunks: HashMap<ChunkPos, Chunk>,
	tick_handler: TickHandler<'a>,
	chunk_generator: WorldGenerator,
}

impl<'a> World<'a> {
	pub fn new() -> World<'a> {
		Self {
			chunk_updates: HashSet::new(),
			chunks: HashMap::new(),
			tick_handler: TickHandler::new(),
			chunk_generator: WorldGenerator::new(69),
		}
	}

	pub fn tick(&mut self) {
		self.tick_handler.tick();
	}

	pub fn poll_chunk(&mut self, pos: &ChunkPos) -> &Chunk {
		return if self.chunks.contains_key(pos) {
			self.chunks.get(pos).unwrap()
		} else {
			let chunk = self.chunk_generator.gen_chunk(pos);
			//let chunk = self.compile_neighbors(chunk, pos);
			// let mut chunk = terraria_gen::generate_terrain(pos);
			self.chunks.insert(*pos, chunk);
			let chunk = self.chunks.get(pos).unwrap();


			// TODO sensei fix this
			unsafe {
				for y in 0..CHUNK_SIZE {
					for x in 0..CHUNK_SIZE {
						let pointer = chunk as *const Chunk as *mut Chunk;
						let self_pointer = self as *const World as *mut World;
						let pos = &WorldPos::from_chunk(pos, x as u8, y as u8);
						(self_pointer.as_mut().unwrap()).update_neighbor(pos, &mut pointer.as_mut().unwrap().solid_tiles[y][x]);
					}
				}
			}

			chunk
		};
	}

	pub fn get<C: NeighborAware>(&self, pos: &WorldPos) -> Option<&C> where Chunk: Grid<C> {
		self.chunks.get(&pos.get_chunk_pos()).map(|chunk| {
			chunk.get(pos.get_chunk_sub_pos())
		})
	}

	pub fn get_mut<C: NeighborAware>(&mut self, pos: &WorldPos) -> Option<&mut C> where Chunk: Grid<C> {
		self.chunks.get_mut(&pos.get_chunk_pos()).map(|chunk| {
			chunk.get_mut(pos.get_chunk_sub_pos())
		})
	}

	pub fn set<C: NeighborAware>(&mut self, pos: &WorldPos, mut object: C) where Chunk: Grid<C> {
		return if self.chunks.contains_key(pos.get_chunk_pos()) {
			self.update_neighbor(pos, &mut object);

			let chunk = self.get_chunk_mut(pos.get_chunk_pos()).unwrap();
			chunk.set(pos.get_chunk_sub_pos(), object);
			self.chunk_updates.insert(pos.get_chunk_pos().clone());
		};
	}

	fn update_neighbor<C: NeighborAware>(&mut self, pos: &WorldPos, object: &mut C) where Chunk: Grid<C> {
		for i in Direction::iter() {
			pos.shift(*i).map(|neighbor_pos| {
				self.chunk_updates.insert(neighbor_pos.get_chunk_pos().clone());
				self.get_mut(&neighbor_pos).map(|neighbor| {
					unsafe {
						// Mutates the values!!!!
						NeighborMatrix::update_neighbor(object, neighbor, *i);
					}
				});
			});
		}
	}

	pub fn get_chunk_mut(&mut self, pos: &ChunkPos) -> Option<&mut Chunk> {
		if self.chunks.contains_key(pos) {
			Some(self.chunks.get_mut(pos).unwrap())
		} else {
			None
		}
	}

	pub fn get_chunk(&self, pos: &ChunkPos) -> Option<&Chunk> {
		if self.chunks.contains_key(pos) {
			Some(self.chunks.get(pos).unwrap())
		} else {
			None
		}
	}
}


pub struct Chunk {
	pos: ChunkPos,
	solid_tiles: [[Tile; CHUNK_SIZE]; CHUNK_SIZE],
	solid_walls: [[Wall; CHUNK_SIZE]; CHUNK_SIZE],
	entities: Vec<Entity>,
}

impl Chunk {
	pub fn new(pos: ChunkPos) -> Chunk {
		let solid_tiles = [[Tile::air(); CHUNK_SIZE]; CHUNK_SIZE];
		let solid_walls = [[Wall::air(); CHUNK_SIZE]; CHUNK_SIZE];

		Self {
			pos,
			solid_tiles,
			solid_walls,
			entities: Vec::new(),
		}
	}
}

impl Grid<Tile> for Chunk {
	fn get_mut(&mut self, pos: &ChunkSubPos) -> &mut Tile {
		&mut self.solid_tiles[pos.y as usize][pos.x as usize]
	}

	fn get(&self, pos: &ChunkSubPos) -> &Tile {
		&self.solid_tiles[pos.y as usize][pos.x as usize]
	}

	fn get_grid(&self) -> &[[Tile; CHUNK_SIZE]; CHUNK_SIZE] {
		&self.solid_tiles
	}

	fn set(&mut self, pos: &ChunkSubPos, child: Tile) {
		self.solid_tiles[pos.y as usize][pos.x as usize] = child;
	}
}

impl Grid<Wall> for Chunk {
	fn get_mut(&mut self, pos: &ChunkSubPos) -> &mut Wall {
		&mut self.solid_walls[pos.y as usize][pos.x as usize]
	}

	fn get(&self, pos: &ChunkSubPos) -> &Wall {
		&self.solid_walls[pos.y as usize][pos.x as usize]
	}

	fn get_grid(&self) -> &[[Wall; CHUNK_SIZE]; CHUNK_SIZE] {
		&self.solid_walls
	}

	fn set(&mut self, pos: &ChunkSubPos, child: Wall) {
		self.solid_walls[pos.y as usize][pos.x as usize] = child;
	}
}


pub trait Grid<C> {
	fn get_mut(&mut self, pos: &ChunkSubPos) -> &mut C;
	fn get(&self, pos: &ChunkSubPos) -> &C;

	fn get_grid(&self) -> &[[C; CHUNK_SIZE]; CHUNK_SIZE];

	fn set(&mut self, pos: &ChunkSubPos, child: C);
}


//	pub fn compile_neighbors(&mut self, mut chunk: Chunk, pos: &ChunkPos) -> Chunk {
// 		// first pass, internals
// 		for y in 0..(CHUNK_SIZE - 1) {
// 			for x in 0..(CHUNK_SIZE - 1) {
// 				// right side
// 				Self::neighbor_update(Direction::Right, &mut chunk, x, y, x + 1, y);
// 				// bottom size
// 				Self::neighbor_update(Direction::Down, &mut chunk, x, y, x, y + 1);
// 			}
// 		}
//
// 		// right chunk border
// 		let x = (CHUNK_SIZE - 1);
// 		let mut common = |chunk: &mut Chunk, y: usize| {
// 			Self::neighbor_update(Direction::Down, chunk, x.clone(), y.clone(), x, y + 1);
// 		};
//
// 		// right side
// 		match self.get_chunk_mut(&pos.right().unwrap()) {
// 			None => {
// 				for y in 0..(CHUNK_SIZE - 1) {
// 					common(&mut chunk, y);
// 				}
// 			}
// 			Some(mut neighbor) => {
// 				for y in 0..(CHUNK_SIZE - 1) {
// 					Self::neighbor_update_c(Direction::Right, &mut chunk, x, y, &mut neighbor, 0, y);
// 					common(&mut chunk, y);
// 				}
// 			}
// 		}
//
//
// 		// bottom chunk border
// 		let y = (CHUNK_SIZE - 1);
// 		let mut common = |chunk: &mut Chunk, x: usize| {
// 			Self::neighbor_update(Direction::Right, chunk, x.clone(), y.clone(), x + 1, y);
// 		};
//
// 		match self.get_chunk_mut(&pos.right().unwrap()) {
// 			None => {
// 				for x in 0..(CHUNK_SIZE - 1) {
// 					common(&mut chunk, x);
// 				};
// 			}
// 			Some(mut neighbor) => {
// 				for x in 0..(CHUNK_SIZE - 1) {
// 					Self::neighbor_update_c(Direction::Down, &mut chunk, x, y, neighbor, x, 0);
// 					common(&mut chunk, x);
// 				};
// 			}
// 		}
//
// 		chunk
// 	}
//
// 	fn neighbor_update_c(dir: Direction, chunk: &mut Chunk, x: usize, y: usize, n_chunk: &mut Chunk, n_x: usize, n_y: usize) {
// 		let chunk_ptr = chunk as *const Chunk as *mut Chunk;
// 		let n_chunk_ptr = n_chunk as *const Chunk as *mut Chunk;
//
// 		// TODO sensei fix this
// 		unsafe {
// 			chunk_ptr.as_mut().unwrap().solid_tiles[y][x].neighbor_update(&dir, &n_chunk_ptr.as_mut().unwrap().solid_tiles[n_y][n_x]);
// 			n_chunk_ptr.as_mut().unwrap().solid_tiles[n_y][n_x].neighbor_update(&dir.flip(), &chunk_ptr.as_mut().unwrap().solid_tiles[y][x]);
// 		}
// 	}
//
// 	fn neighbor_update(dir: Direction, chunk: &mut Chunk, x: usize, y: usize, n_x: usize, n_y: usize) {
// 		let chunk_ptr = chunk as *const Chunk as *mut Chunk;
//
// 		// TODO sensei fix this
// 		unsafe {
// 			chunk_ptr.as_mut().unwrap().solid_tiles[y][x].neighbor_update(&dir, &chunk_ptr.as_mut().unwrap().solid_tiles[n_y][n_x]);
// 			chunk_ptr.as_mut().unwrap().solid_tiles[n_y][n_x].neighbor_update(&dir.flip(), &chunk_ptr.as_mut().unwrap().solid_tiles[y][x]);
// 		}
// 	}
