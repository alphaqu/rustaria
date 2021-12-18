#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RenderType {
    /// Does not render things behind. Transparency is off
    Solid,
    /// Does render stuff behind. Transparency is on
    Opaque,
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