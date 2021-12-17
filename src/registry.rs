#[derive(Copy, Clone)]
pub enum RenderType {
    /// Does not render things behind. Transparency is off
    Solid,
    /// Does render stuff behind. Transparency is on
    Opaque,
    /// Skips creating the quad and fully ignores rendering.
    Transparent,
}

#[derive(Copy, Clone)]
pub struct TileId {
    id: u32,
}

#[derive(Copy, Clone)]
pub struct WallId {
    id: u32,
}

pub struct Fluid {}

// TODO entities
pub struct Entity {}