use std::collections::{HashMap, HashSet};

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
const RENDER_DISTANCE: i32 = 16;

pub struct World {
	players: Vec<Player>,
	pub chunk_updates: HashSet<ChunkPos>,
	chunks: HashMap<ChunkPos, Chunk>,
	chunk_generator: WorldGenerator,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct PlayerId {
	id: usize,
}

impl Default for PlayerId {
	fn default() -> Self {
		Self {
			id: usize::MAX
		}
	}
}

impl Default for World {
	fn default() -> World {
		Self {
			players: Vec::new(),
			chunk_updates: HashSet::new(),
			chunks: HashMap::new(),
			chunk_generator: WorldGenerator::new(69),
		}
	}
}

impl World {
	// FIXME Not multiplayer ready because if a player leaves the ids will be misaligned
	pub fn player_join(&mut self, player: Player) -> PlayerId {
		let id = PlayerId {
			id: self.players.len()
		};
		self.players.insert(id.id, player);
		id
	}

	pub fn acquire_player_mut(&mut self, id: &PlayerId) -> &mut Player {
		self.players.get_mut(id.id).expect("Could not find player")
	}

	#[profiler_macro::profile]
	pub fn acquire_player(&self, id: &PlayerId) -> &Player {
		let ret = self.players.get(id.id).expect("Could not find player");
		ret
	}


	#[profiler_macro::profile]
	pub fn tick_world(&mut self) {
		self.tick_physics();
		self.tick_chunks();
		self.poll_chunks();
	}

	#[profiler_macro::profile]
	fn poll_chunks(&mut self) {
		if let Some(new_chunks) = self.chunk_generator.generate_chunks() {
			self.add_chunks(new_chunks)
		}
	}

	#[profiler_macro::profile]
	fn add_chunks(&mut self, new_chunks: Vec<(ChunkPos, Chunk)>) {
		for (pos, chunk) in new_chunks {
			self.chunks.insert(pos, chunk);
			let chunk = self.chunks.get(&pos).unwrap();
			self.update_borders::<Tile>(&pos, chunk);
			self.update_borders::<Wall>(&pos, chunk);
			for dir in Direction::iter() {
				if let Some(neighbor) = pos.shift(dir) {
					self.chunk_updates.insert(neighbor);
				}
			}
		}
	}

	#[profiler_macro::profile]
	fn tick_chunks(&mut self) {
		for player in &self.players {
			let lookup_pos = ChunkPos::from_player(player);

			for x in (-RENDER_DISTANCE)..RENDER_DISTANCE {
				if let Some(lookup_pos) = lookup_pos.shift_amount(Direction::Left, x) {
					for y in (-RENDER_DISTANCE)..RENDER_DISTANCE {
						if let Some(pos) = lookup_pos.shift_amount(Direction::Down, y) {
							if !self.chunks.contains_key(&pos) {
								self.chunk_generator.add_chunk(&pos);
							}
						}
					}
				}
			}
		}
	}

	#[profiler_macro::profile]
	fn tick_physics(&mut self) {
		for player in &mut self.players {
			player.pos_x += player.vel_x * player.speed;
			player.pos_y += player.vel_y * player.speed;
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
									dir
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
									dir
								);
							}
						}
					}
				})
			});
		}
	}

	pub fn get<C: NeighborAware>(&self, pos: &WorldPos) -> Option<&C> where Chunk: Grid<C> {
		self.chunks.get(pos.get_chunk_pos()).map(|chunk| {
			chunk.get(pos.get_chunk_sub_pos())
		})
	}

	pub fn get_mut<C: NeighborAware>(&mut self, pos: &WorldPos) -> Option<&mut C> where Chunk: Grid<C> {
		self.chunks.get_mut(pos.get_chunk_pos()).map(|chunk| {
			chunk.get_mut(pos.get_chunk_sub_pos())
		})
	}

	pub fn set<C: NeighborAware>(&mut self, pos: &WorldPos, mut object: C) where Chunk: Grid<C> {
		return if self.chunks.contains_key(pos.get_chunk_pos()) {
			self.update_neighbor(pos, &mut object);

			let chunk = self.get_chunk_mut(pos.get_chunk_pos()).unwrap();
			chunk.set(pos.get_chunk_sub_pos(), object);
			self.chunk_updates.insert(*pos.get_chunk_pos());
		};
	}


	#[profiler_macro::profile]
	fn update_neighbor<C: NeighborAware>(&mut self, pos: &WorldPos, object: &mut C) where Chunk: Grid<C> {
		for i in Direction::iter() {
			if let Some(neighbor_pos) = pos.shift(i) {
				self.chunk_updates.insert(*neighbor_pos.get_chunk_pos());

				if let Some(neighbor) = self.get_mut(&neighbor_pos) {
					unsafe {
						// Mutates the values!!!
						NeighborMatrix::update_neighbor(object, neighbor, i);
					}
				}
			}
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

impl Default for Chunk {
	fn default() -> Self {
		let solid_tiles = [[Tile::air(); CHUNK_SIZE]; CHUNK_SIZE];
		let solid_walls = [[Wall::air(); CHUNK_SIZE]; CHUNK_SIZE];

		Self {
			solid_tiles,
			solid_walls,
		}
	}
}

macro_rules! grid {
    ($($TYPE:ty => $ARR:ident),*) => {
		$(
		impl Grid<$TYPE> for Chunk {
			fn get_mut(&mut self, pos: &ChunkSubPos) -> &mut $TYPE {
				&mut self.$ARR[pos.y as usize][pos.x as usize]
			}

			fn get(&self, pos: &ChunkSubPos) -> &$TYPE {
				&self.$ARR[pos.y as usize][pos.x as usize]
			}

			fn get_grid(&self) -> &[[$TYPE; CHUNK_SIZE]; CHUNK_SIZE] {
				&self.$ARR
			}

			fn get_grid_mut(&mut self) -> &mut [[$TYPE; CHUNK_SIZE]; CHUNK_SIZE] {
				&mut self.$ARR
			}

			fn set(&mut self, pos: &ChunkSubPos, child: $TYPE) {
				self.$ARR[pos.y as usize][pos.x as usize] = child;
			}
		}
		)*
	};
}

grid!(
	Tile => solid_tiles,
	Wall => solid_walls
);

pub trait Grid<C> {
	fn get_mut(&mut self, pos: &ChunkSubPos) -> &mut C;
	fn get(&self, pos: &ChunkSubPos) -> &C;

	fn get_grid(&self) -> &[[C; CHUNK_SIZE]; CHUNK_SIZE];
	fn get_grid_mut(&mut self) -> &mut [[C; CHUNK_SIZE]; CHUNK_SIZE];

	fn set(&mut self, pos: &ChunkSubPos, child: C);
}