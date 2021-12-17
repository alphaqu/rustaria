use crate::registry::{TileId};
use crate::world::tick::{Tickable, TickResult};

#[derive(Copy, Clone)]
pub enum CollisionType {
    CollidesPlayer,
    Nothing,
}

pub struct Tile {
    id: TileId,
}

//pub trait Tile: Tickable {
//    fn get_id(&self) -> TileId;
//    fn get_type(&self) -> RenderType;
//    fn get_collision(&self) -> CollisionType;
//}

//struct BasicTile {
//    id: TileId,
//    render_type: RenderType,
//    collision_type: CollisionType,
//}
//
//impl Tile for BasicTile {
//    fn get_id(&self) -> TileId {
//        self.id
//    }
//
//    fn get_type(&self) -> RenderType {
//        self.render_type
//    }
//
//    fn get_collision(&self) -> CollisionType {
//        self.collision_type
//    }
//}

impl Tickable for Tile {
    fn tick(&self) -> TickResult {
        panic!("Cannot tick basic tile")
    }
}