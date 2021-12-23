use crate::pos::{ChunkPos, WorldPos};
use crate::world::{Chunk, Grid};
use crate::world::neighbor::NeighborAware;

pub trait WorldView {
	/// Tries to get the chunk. In this case it will generate a new chunk if it does not exist.
	/// If you just want fast access use `request_chunk` instead
	fn get_chunk(&mut self, pos: &ChunkPos) -> Option<&Chunk>;

	/// Gets the chunk if it exists. Else returns `None`
	fn request_chunk(&self, pos: &ChunkPos) -> Option<&Chunk>;

	/// Gets a single `NeighborAware` object
	fn get<C: NeighborAware>(&self, pos: &WorldPos) -> Option<&C>;
}