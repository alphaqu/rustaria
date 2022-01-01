pub struct ClientSettings {
	pub zoom: f32,
	pub render_distance: u32,
}

impl ClientSettings {
	pub fn new() -> Self {
		Self {
			zoom: 1.1f32,
			render_distance: 16,
		}
	}
}
