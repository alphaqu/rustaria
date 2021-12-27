use crate::misc::util::CallbackResponse;
use crate::settings::Settings;
use crate::world::tick::Tickable;

const GRAVITY: f32 = 0.2;
const BASE_SPEED: f32 = 8f32;

pub struct Player {
	pub speed: f32,
	pub pos_x: f32,
	pub pos_y: f32,
	pub vel_x: f32,
	pub vel_y: f32,
}

impl Player {
	pub fn new() -> Player {
		Self {
			speed: BASE_SPEED,
			pos_x: 0.0,
			pos_y: 0.0,
			vel_x: 0.0,
			vel_y: 0.0,
		}
	}

	pub fn tick(&mut self) {

	}
}
