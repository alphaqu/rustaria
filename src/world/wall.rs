use crate::consts;
use crate::consts::WallId;
use crate::util::Direction;
use crate::world::neighbor::{NeighborAware, NeighborMatrix, NeighborType};
use crate::world::tile::Tile;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Wall {
	pub id: WallId,
	pub variance: u8,
	matrix: NeighborMatrix,
}

impl Wall {
	pub fn air() -> Wall {
		Self {
			variance: 0,
			id: WallId { id: 0 },
			matrix: NeighborMatrix::new(),
		}
	}

	pub fn id(id: WallId) -> Wall {
		Self {
			variance: 0,
			id,
			matrix: NeighborMatrix::new(),
		}
	}

	pub fn get_id(&self) -> &WallId {
		&self.id
	}
}

impl NeighborAware for Wall {
	fn get_neighbor_matrix(&self) -> &NeighborMatrix {
		&self.matrix
	}

	fn apply_neighbor(&self, neighbor: &Wall) -> NeighborType {
		if neighbor.id == consts::WALL_AIR {
			NeighborType::Air
		} else /*if neighbor.id == target.id*/ {
			NeighborType::Same
		}
	}
}

