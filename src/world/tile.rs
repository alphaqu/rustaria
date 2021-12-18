use crate::registry::TileId;
use crate::util::CallbackResponse;
use crate::world::tick::Tickable;

#[derive(Copy, Clone)]
pub enum CollisionType {
    CollidesPlayer,
    Nothing,
}

pub struct Tile {
    id: TileId,
}

impl Tickable for Tile {
    fn tick(&self) -> CallbackResponse {
        panic!("Cannot tick basic tile")
    }
}