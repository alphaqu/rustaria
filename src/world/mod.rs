use std::collections::{HashMap, HashSet};
use std::time::Instant;

use neighbor::NeighborAware;

use crate::gen::WorldGenerator;
use crate::misc::pos::{ChunkPos, ChunkSubPos, WorldPos};
use crate::misc::util::{CHUNK_SIZE, Direction};
use crate::Player;
use crate::world::neighbor::NeighborMatrix;
use crate::world::tile::Tile;
use crate::world::wall::Wall;

pub mod tile;
pub mod wall;
pub mod tick;
pub mod neighbor;

// un hard code this
const RENDER_DISTANCE: i32 = 32;

pub struct World {
	players: HashMap<PlayerId, Player>,
	pub chunk_updates: HashSet<ChunkPos>,
	chunks: HashMap<ChunkPos, Chunk>,
	chunk_generator: WorldGenerator,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct PlayerId {
	id: usize,
}

impl PlayerId {
	pub fn new() -> Self {
		Self {
			id: usize::MAX
		}
	}
}

impl World {
	pub fn new() -> World {
		Self {
			players: HashMap::new(),
			chunk_updates: HashSet::new(),
			chunks: HashMap::new(),
			chunk_generator: WorldGenerator::new(69),
		}
	}

	// FIXME Not multiplayer ready because if a player leaves the ids will be misaligned
	pub fn player_join(&mut self, player: Player) -> PlayerId {
		let id = PlayerId {
			id: self.players.len()
		};
		self.players.insert(id, player);
		id
	}

	pub fn acquire_player_mut(&mut self, id: &PlayerId) -> &mut Player {
		self.players.get_mut(id).expect("Could not find player")
	}

	pub fn acquire_player(&self, id: &PlayerId) -> &Player {
		self.players.get(id).expect("Could not find player")
	}


	pub fn tick(&mut self) {
		for (_, player) in &mut self.players {
			player.pos_x += player.vel_x * player.speed;
			player.pos_y += player.vel_y * player.speed;
		}

		for (_, player) in &self.players {
			let lookup_pos = ChunkPos::from_player(player);

			for x in (-RENDER_DISTANCE)..RENDER_DISTANCE {
				if let Some(lookup_pos) = lookup_pos.shift_amount(&Direction::Left, x) {
					for y in (-RENDER_DISTANCE)..RENDER_DISTANCE {
						if let Some(pos) = lookup_pos.shift_amount(&Direction::Down, y) {
							if !self.chunks.contains_key(&pos) {
								self.chunk_generator.add_chunk(&pos);
							}
						}
					}
				}
			}
		}


		let start = Instant::now();
		if let Some(new_chunks) = self.chunk_generator.generate_chunks() {
			let length = new_chunks.len();
			for (pos, chunk) in new_chunks {
				self.chunks.insert(pos.clone(), chunk);
				let chunk = self.chunks.get(&pos).unwrap();
				self.update_borders::<Tile>(&pos, chunk);
				self.update_borders::<Wall>(&pos, chunk);
				for dir in Direction::iter() {
					pos.shift(dir).map(|neighbor| {
						self.chunk_updates.insert(neighbor);
					});
				}
//
				//// TODO sensei fix this
				//unsafe {
				//	for y in 0..CHUNK_SIZE {
				//		for x in 0..CHUNK_SIZE {
				//			let pointer = chunk as *const Chunk as *mut Chunk;
				//			let self_pointer = self as *const World as *mut World;
				//			let pos = &WorldPos::from_chunk(&pos, x as u8, y as u8);
				//			(self_pointer.as_mut().unwrap()).update_neighbor(pos, &mut pointer.as_mut().unwrap().solid_tiles[y][x]);
				//		}
				//	}
				//}
			}
			//println!("Generated {} chunks in {}ms", length, start.elapsed().as_millis());
		}
	}

	fn update_borders<C: NeighborAware>(&self, pos: &ChunkPos, chunk: &Chunk) where Chunk: Grid<C> {
		for dir in Direction::iter() {
			pos.shift(dir).map(|neighbor_pos| {
				self.chunks.get(&neighbor_pos).map(|neighbor| {
					if dir.is_vertical() {
						let source = dir.get_y_border();
						let neigh = dir.flip().get_y_border();
						for x in 0..CHUNK_SIZE {
							unsafe {
								NeighborMatrix::update_neighbor(
									chunk.get(&ChunkSubPos::new(x as u8, source)),
									neighbor.get(&ChunkSubPos::new(x as u8, neigh)),
									dir.clone()
								);
							}
						}
					} else {
						let source = dir.get_x_border();
						let neigh = dir.flip().get_x_border();
						for y in 0..CHUNK_SIZE {
							unsafe {
								NeighborMatrix::update_neighbor(
									chunk.get(&ChunkSubPos::new(source, y as u8)),
									neighbor.get(&ChunkSubPos::new(neigh, y as u8)),
									dir.clone()
								);
							}
						}
					}
				})
			});
		}
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
	solid_tiles: [[Tile; CHUNK_SIZE]; CHUNK_SIZE],
	solid_walls: [[Wall; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
	pub fn new() -> Chunk {
		let solid_tiles = [[Tile::air(); CHUNK_SIZE]; CHUNK_SIZE];
		let solid_walls = [[Wall::air(); CHUNK_SIZE]; CHUNK_SIZE];

		Self {
			solid_tiles,
			solid_walls,
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

	fn get_grid_mut(&mut self) -> &mut [[Tile; CHUNK_SIZE]; CHUNK_SIZE] {
		&mut self.solid_tiles
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

	fn get_grid_mut(&mut self) -> &mut [[Wall; CHUNK_SIZE]; CHUNK_SIZE] {
		&mut self.solid_walls
	}

	fn set(&mut self, pos: &ChunkSubPos, child: Wall) {
		self.solid_walls[pos.y as usize][pos.x as usize] = child;
	}
}


pub trait Grid<C> {
	fn get_mut(&mut self, pos: &ChunkSubPos) -> &mut C;
	fn get(&self, pos: &ChunkSubPos) -> &C;

	fn get_grid(&self) -> &[[C; CHUNK_SIZE]; CHUNK_SIZE];
	fn get_grid_mut(&mut self) -> &mut [[C; CHUNK_SIZE]; CHUNK_SIZE];

	fn set(&mut self, pos: &ChunkSubPos, child: C);
}


//	pub fn compile_neighbors(&mut self, mut render: Chunk, pos: &ChunkPos) -> Chunk {
// 		// first pass, internals
// 		for y in 0..(CHUNK_SIZE - 1) {
// 			for x in 0..(CHUNK_SIZE - 1) {
// 				// right side
// 				Self::neighbor_update(Direction::Right, &mut render, x, y, x + 1, y);
// 				// bottom size
// 				Self::neighbor_update(Direction::Down, &mut render, x, y, x, y + 1);
// 			}
// 		}
//
// 		// right render border
// 		let x = (CHUNK_SIZE - 1);
// 		let mut common = |render: &mut Chunk, y: usize| {
// 			Self::neighbor_update(Direction::Down, render, x.clone(), y.clone(), x, y + 1);
// 		};
//
// 		// right side
// 		match self.get_chunk_mut(&pos.right().unwrap()) {
// 			None => {
// 				for y in 0..(CHUNK_SIZE - 1) {
// 					common(&mut render, y);
// 				}
// 			}
// 			Some(mut neighbor) => {
// 				for y in 0..(CHUNK_SIZE - 1) {
// 					Self::neighbor_update_c(Direction::Right, &mut render, x, y, &mut neighbor, 0, y);
// 					common(&mut render, y);
// 				}
// 			}
// 		}
//
//
// 		// bottom render border
// 		let y = (CHUNK_SIZE - 1);
// 		let mut common = |render: &mut Chunk, x: usize| {
// 			Self::neighbor_update(Direction::Right, render, x.clone(), y.clone(), x + 1, y);
// 		};
//
// 		match self.get_chunk_mut(&pos.right().unwrap()) {
// 			None => {
// 				for x in 0..(CHUNK_SIZE - 1) {
// 					common(&mut render, x);
// 				};
// 			}
// 			Some(mut neighbor) => {
// 				for x in 0..(CHUNK_SIZE - 1) {
// 					Self::neighbor_update_c(Direction::Down, &mut render, x, y, neighbor, x, 0);
// 					common(&mut render, x);
// 				};
// 			}
// 		}
//
// 		render
// 	}
//
// 	fn neighbor_update_c(dir: Direction, render: &mut Chunk, x: usize, y: usize, n_chunk: &mut Chunk, n_x: usize, n_y: usize) {
// 		let chunk_ptr = render as *const Chunk as *mut Chunk;
// 		let n_chunk_ptr = n_chunk as *const Chunk as *mut Chunk;
//
// 		// TODO sensei fix this
// 		unsafe {
// 			chunk_ptr.as_mut().unwrap().solid_tiles[y][x].neighbor_update(&dir, &n_chunk_ptr.as_mut().unwrap().solid_tiles[n_y][n_x]);
// 			n_chunk_ptr.as_mut().unwrap().solid_tiles[n_y][n_x].neighbor_update(&dir.flip(), &chunk_ptr.as_mut().unwrap().solid_tiles[y][x]);
// 		}
// 	}
//
// 	fn neighbor_update(dir: Direction, render: &mut Chunk, x: usize, y: usize, n_x: usize, n_y: usize) {
// 		let chunk_ptr = render as *const Chunk as *mut Chunk;
//
// 		// TODO sensei fix this
// 		unsafe {
// 			chunk_ptr.as_mut().unwrap().solid_tiles[y][x].neighbor_update(&dir, &chunk_ptr.as_mut().unwrap().solid_tiles[n_y][n_x]);
// 			chunk_ptr.as_mut().unwrap().solid_tiles[n_y][n_x].neighbor_update(&dir.flip(), &chunk_ptr.as_mut().unwrap().solid_tiles[y][x]);
// 		}
// 	}
