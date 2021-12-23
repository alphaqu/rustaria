use crate::consts;
use crate::consts::TileId;
use crate::util::{CallbackResponse, Direction};
use crate::world::neighbor::{NeighborAware, NeighborMatrix, NeighborType};
use crate::world::tick::Tickable;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Tile {
	pub id: TileId,
	pub variance: u8,
	pub matrix: NeighborMatrix
}

impl Tile {
	pub fn air() -> Tile {
		Self {
			variance: 0,
			id: TileId { id: 0 },
			matrix: NeighborMatrix::new()
		}
	}

	pub fn id(id: TileId) -> Tile {
		Self {
			id,
			variance: 0,
			matrix: NeighborMatrix::new()
		}
	}

	pub fn get_id(&self) -> &TileId {
		&self.id
	}
}

impl Tickable for Tile {
	fn tick(&self) -> CallbackResponse {
		panic!("Cannot tick basic tile")
	}
}

impl NeighborAware for Tile {
	fn get_neighbor_matrix(&self) -> &NeighborMatrix {
		&self.matrix
	}

	fn apply_neighbor(&self, neighbor: &Tile) -> NeighborType {
		if neighbor.id == consts::TILE_AIR {
			NeighborType::Air
		} else /*if neighbor.id == target.id*/ {
			NeighborType::Same
		}
	}
}
