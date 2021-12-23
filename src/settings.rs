
pub struct Settings {
    pub render_distance: u16,
    pub zoom: f32,
    pub cull_chunks: bool,
}

impl Settings {
    pub fn new() -> Settings {
        Self {
            render_distance: 4,
            zoom: 1 as f32,
            cull_chunks: true
        }
    }
}