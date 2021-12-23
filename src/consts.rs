macro_rules! pub_const_tile_id {
    ($($NAME:ident $ID:expr;)*) => {
        $(pub const $NAME: TileId = TileId { id: $ID };)*
    };
}

macro_rules! pub_const_wall_id {
    ($($NAME:ident $ID:expr;)*) => {
        $(pub const $NAME: WallId = WallId { id: $ID };)*
    };
}

/// https://terraria.fandom.com/wiki/Tile_IDs
/// Tile ids
pub_const_tile_id! {
    TILE_AIR 0;
    TILE_DIRT 1;
    TILE_STONE 2;
    TILE_GRASS 3;
    TILE_IRON_ORE 7;
    TILE_COPPER_ORE 8;
    TILE_GOLD_ORE 9;
    TILE_SILVER_ORE 10;
    TILE_DEMONITE_ORE 23;
    TILE_CORRUPT_GRASS 23;
    TILE_EBONSTONE 23;
    TILE_WOOD 31;
    TILE_METEORITE 38;
    TILE_GRAY_BRICK 39;
    TILE_RED_BRICK 40;
    // clay is an imposter brick. very sus
    TILE_CLAY_BLOCK 41;
    TILE_BLUE_BRICK 42;
    TILE_GREEN_BRICK 44;
    TILE_PINK_BRICK 45;
    TILE_GOLD_BRICK 46;
    TILE_SILVER_BRICK 47;
    TILE_COPPER_BRICK 48;
    TILE_SPIKE 49;
    TILE_COBWEB 52;
    TILE_REGULAR_VINE 53;
    TILE_SAND 54;
    TILE_GLASS 55;
    TILE_OBSIDIAN 57;
    TILE_ASH_BLOCK 58;
    TILE_HELLSTONE 59;
    TILE_MUD_BLOCK 60;
    TILE_JUNGLE_GRASS 61;
    TILE_JUNGLE_VINE 63;
    // mage shit
    TILE_SAPPHIRE 64;
    TILE_RUBY 65;
    TILE_EMERALD 66;
    TILE_TOPAZ 67;
    TILE_AMETHYST 68;
    TILE_DIAMOND 69;
    // haha funny
    TILE_JUNGLE_THORNY_BUSH 70;
    TILE_MUSHROOM_GRASS 71;
    // hell stuff
    TILE_OBSIDIAN_BRICK 76;
    TILE_HELLSTONE_BRICK 77;
    // oo hardcore world
    TILE_COBALT_ORE 108;
    TILE_MYTHRIL_ORE 109;
    TILE_HALLOWED_GRASS 110;
    TILE_ADAMANTITE_ORE 112;
    TILE_EBONSAND_BLOCK 113;
    TILE_PEARLSAND_BLOCK 117;
    TILE_PEARLSTONE_BRICK 119;
    TILE_IRIDESCENT_BRICK 120;
    TILE_MUDSTONE_BRICK 121;
    TILE_COBALT_BRICK 122;
    TILE_MYTHRIL_BRICK 123;
    TILE_SILT_BLOCK 124;
    TILE_WOOD_BEAM 125;
    TILE_ICE_BLOCK 129;
    TILE_DEMONITE_BRICK 141;
    TILE_CANDY_CANE_BLOCK 146;
    TILE_GREEN_CANDY_CANE_BLOCK 147;
    TILE_SNOW_BLOCK 148;
    TILE_SNOW_BRICK 149;
}

pub_const_wall_id! {
	WALL_AIR 0;
    WALL_STONE 1;
    WALL_DIRT 2;
    WALL_EBON_STONE 3;
    WALL_WOOD 4;
    WALL_GRAY_BRICK 5;
    WALL_RED_BRICK 6;
    WALL_BLUE_DUNGEON 7;
    WALL_GREEN_DUNGEON 8;
    WALL_PINK_DUNGEON 9;
    WALL_GOLD_BRICK 10;
    WALL_SILVER_BRICK 11;
}
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RenderType {
	/// Does not render things behind. Transparency is off
	Opaque,
	/// Does render stuff behind. Transparency is on
	Solid,
	/// Skips creating the quad and fully ignores rendering.
	Transparent,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct TileId {
	pub id: u32,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct WallId {
	pub id: u32,
}

pub struct Fluid {}

// TODO entities
pub struct Entity {}
