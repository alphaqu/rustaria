use crate::client::opengl::gl;

const TILE_SCALE: f32 = 20.0;

pub struct Viewport {
    pub width: i32,
    pub height: i32,
    pub gl_tile_width: f32,
    pub gl_tile_height: f32,
}

impl Viewport {
    pub fn new(width: i32, height: i32) -> Viewport {
        let mut viewport = Self { width: 0, height: 0, gl_tile_width: 0.0, gl_tile_height: 0.0 };
        viewport.update_size(width, height);
        viewport
    }

    pub fn update_size(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
        self.calc_tile_size(width, height);
        gl::viewport(0, 0, width, height);
    }

    fn calc_tile_size(&mut self, width: i32, height: i32) {
        self.gl_tile_width = 1f32 / TILE_SCALE;
        self.gl_tile_height = ((width as f32) / height as f32) / TILE_SCALE;
    }
}