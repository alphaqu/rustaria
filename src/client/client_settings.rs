pub struct ClientSettings {
	zoom: f32,
	render_distance: u32,
	chunk_culling: bool,
}

impl ClientSettings {
	pub fn new() -> Self {
		Self {
			zoom: 0.0,
			render_distance: 0,
			chunk_culling: false
		}
	}
}
