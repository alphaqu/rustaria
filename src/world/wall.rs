use crate::world::neighbor::{NeighborAware, NeighborMatrix, NeighborType};
use crate::world::wall;

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
			matrix: NeighborMatrix::default(),
		}
	}

	pub fn id(id: WallId) -> Wall {
		Self {
			variance: 0,
			id,
			matrix: NeighborMatrix::default(),
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
		if neighbor.id == wall::AIR {
			NeighborType::Air
		} else /*if neighbor.id == target.id*/ {
			NeighborType::Same
		}
	}
}

macro_rules! pub_const_wall_id {
    ($($NAME:ident $ID:expr;)*) => {
        $(pub const $NAME: WallId = WallId { id: $ID };)*
    };
}

pub_const_wall_id! {
	AIR 0;
    STONE 1;
    DIRT 2;
    EBON_STONE 3;
    WOOD 4;
    GRAY_BRICK 5;
    RED_BRICK 6;
    BLUE_DUNGEON 7;
    GREEN_DUNGEON 8;
    PINK_DUNGEON 9;
    GOLD_BRICK 10;
    SILVER_BRICK 11;
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct WallId {
	pub id: u32,
}
