pub struct InputHandler {

}


pub trait Event {
	fn pressed(&mut self, meta: &EventMetadata, intensity: u8);
	fn released(&mut self, meta: &EventMetadata, intensity: u8);
}

pub struct EventMetadata {
	mouse_x: i32,
	mouse_y: i32,
}