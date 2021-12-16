#![allow(dead_code)]
#![allow(unused_variables)]

#[derive(Copy, Clone)]
pub struct Tile {
    pub id: Identifier
}

#[derive(Copy, Clone)]
pub struct TileNeighborData {
    top_neighbor: TileNeighborType,
    down_neighbor: TileNeighborType,
    left_neighbor: TileNeighborType,
    right_neighbor: TileNeighborType,
}

#[derive(Copy, Clone)]
pub enum TileNeighborType {
    Air,
    Same,
    Transitional,
}

#[derive(Copy, Clone)]
pub struct Chunk {
    pub chunk_x: i32,
    pub chunk_y: u16,
    pub tile_data: [[Tile; 8]; 8],
}

impl Chunk {
    pub fn parse_debug(chunk_x: i32, chunk_y: u16, values: [[u32; 8]; 8]) -> Chunk {
        let mut x = 0;
        let mut y = 0;
        let empty = Tile { id: Identifier { id: 0 } };
        let mut tile_data: [[Tile; 8]; 8] = [
            [empty,empty,empty,empty,empty,empty,empty,empty],
            [empty,empty,empty,empty,empty,empty,empty,empty],
            [empty,empty,empty,empty,empty,empty,empty,empty],
            [empty,empty,empty,empty,empty,empty,empty,empty],
            [empty,empty,empty,empty,empty,empty,empty,empty],
            [empty,empty,empty,empty,empty,empty,empty,empty],
            [empty,empty,empty,empty,empty,empty,empty,empty],
            [empty,empty,empty,empty,empty,empty,empty,empty]
        ];
        for y in 0..values.len() {
            let y_row = values[y];
            for x in 0..y_row.len() {
                tile_data[y][x] = Tile { id: Identifier { id: y_row[x] } };
            }
        }
        Self {
            chunk_x,
            chunk_y,
            tile_data
        }
    }
}

#[derive(Copy, Clone)]
pub struct Identifier {
   pub id: u32,
}

#[derive(Copy, Clone)]
struct TileEntityId {
    entity: u32,
}