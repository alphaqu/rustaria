use crate::util::{CallbackResponse, Direction};
use crate::world::neighbor::{NeighborAware, NeighborMatrix, NeighborType};
use crate::world::tick::Tickable;
use crate::world::tile;

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
		if neighbor.id == tile::AIR {
			NeighborType::Air
		} else /*if neighbor.id == target.id*/ {
			NeighborType::Same
		}
	}
}

macro_rules! pub_const_tile_id {
    ($($NAME:ident $ID:expr;)*) => {
        $(pub const $NAME: TileId = TileId { id: $ID };)*
    };
}

/// https://terraria.fandom.com/wiki/Tile_IDs
/// Tile ids
pub_const_tile_id! {
    AIR 0;
    DIRT 1;
    STONE 2;
    GRASS 3;
    IRON_ORE 7;
    COPPER_ORE 8;
    GOLD_ORE 9;
    SILVER_ORE 10;
    DEMONITE_ORE 23;
    CORRUPT_GRASS 23;
    EBONSTONE 23;
    WOOD 31;
    METEORITE 38;
    GRAY_BRICK 39;
    RED_BRICK 40;
    // clay is an imposter brick. very sus
    CLAY_BLOCK 41;
    BLUE_BRICK 42;
    GREEN_BRICK 44;
    PINK_BRICK 45;
    GOLD_BRICK 46;
    SILVER_BRICK 47;
    COPPER_BRICK 48;
    SPIKE 49;
    COBWEB 52;
    REGULAR_VINE 53;
    SAND 54;
    GLASS 55;
    OBSIDIAN 57;
    ASH_BLOCK 58;
    HELLSTONE 59;
    MUD_BLOCK 60;
    JUNGLE_GRASS 61;
    JUNGLE_VINE 63;
    // mage shit
    SAPPHIRE 64;
    RUBY 65;
    EMERALD 66;
    TOPAZ 67;
    AMETHYST 68;
    DIAMOND 69;
    // haha funny
    JUNGLE_THORNY_BUSH 70;
    MUSHROOM_GRASS 71;
    // hell stuff
    OBSIDIAN_BRICK 76;
    HELLSTONE_BRICK 77;
    // oo hardcore world
    COBALT_ORE 108;
    MYTHRIL_ORE 109;
    HALLOWED_GRASS 110;
    ADAMANTITE_ORE 112;
    EBONSAND_BLOCK 113;
    PEARLSAND_BLOCK 117;
    PEARLSTONE_BRICK 119;
    IRIDESCENT_BRICK 120;
    MUDSTONE_BRICK 121;
    COBALT_BRICK 122;
    MYTHRIL_BRICK 123;
    SILT_BLOCK 124;
    WOOD_BEAM 125;
    ICE_BLOCK 129;
    DEMONITE_BRICK 141;
    CANDY_CANE_BLOCK 146;
    GREEN_CANDY_CANE_BLOCK 147;
    SNOW_BLOCK 148;
    SNOW_BRICK 149;
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct TileId {
	pub id: u32,
}
