use crate::player::Player;
use crate::util::{CHUNK_SIZE, Direction};

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct WorldPos {
	chunk_pos: ChunkPos,
	chunk_sub_pos: ChunkSubPos,
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct ChunkPos {
	pub x: i16,
	pub y: u16,
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct ChunkSubPos {
	pub x: u8,
	pub y: u8,
}

impl WorldPos {
	pub fn new(mut x: i32, y: u32) -> WorldPos {
		let chunk_x = x.div_euclid(CHUNK_SIZE as i32) as i16;
		let chunk_y = y.div_euclid(CHUNK_SIZE as u32) as u16;

		let local_x = ((x as i32).rem_euclid(CHUNK_SIZE as i32)) as u8;
		let local_y = ((y as u32).rem_euclid(CHUNK_SIZE as u32)) as u8;
		Self {
			chunk_pos: ChunkPos::new(chunk_x, chunk_y),
			chunk_sub_pos: ChunkSubPos::new(local_x, local_y),
		}
	}

	pub fn from_chunk(chunk_pos: &ChunkPos, x: u8, y: u8) -> WorldPos {
		Self {
			chunk_pos: chunk_pos.clone(),
			chunk_sub_pos: ChunkSubPos::new(x, y),
		}
	}

	pub fn get_chunk_pos(&self) -> &ChunkPos {
		&self.chunk_pos
	}

	pub fn get_chunk_sub_pos(&self) -> &ChunkSubPos {
		&self.chunk_sub_pos
	}

	pub fn shift(&self, direction: Direction) -> Option<WorldPos> {
		if self.chunk_sub_pos.is_border(&direction) {
			let chunk_sub_pos = self.chunk_sub_pos.shift_overlooping(direction);
			self.chunk_pos.shift(&direction).map(|chunk_pos| WorldPos { chunk_pos, chunk_sub_pos })
		} else {
			self.chunk_sub_pos.shift(direction).map(|chunk_sub_pos| {
				WorldPos { chunk_pos: self.chunk_pos, chunk_sub_pos }
			})
		}
	}
}

impl ChunkPos {
	pub fn new(x: i16, y: u16) -> ChunkPos {
		Self {
			x,
			y,
		}
	}

	pub fn from_player(player: &Player) -> ChunkPos {
		let chunk_x = (player.pos_x as f32 / CHUNK_SIZE as f32).floor() as i16;
		let chunk_y = (player.pos_y as f32 / CHUNK_SIZE as f32).floor() as u16;
		ChunkPos::new(chunk_x, chunk_y)
	}

	pub fn shift(&self, direction: &Direction) -> Option<ChunkPos> {
		let next_x = (self.x as i32 + direction.get_x_difference() as i32);
		let next_y = (self.y as i32 + direction.get_y_difference() as i32);
		if next_y <= u16::MIN as i32 || next_y >= u16::MAX as i32 || next_x <= i16::MIN as i32 || next_x >= i16::MAX as i32 {
			return None;
		}
		let mut pos = self.clone();
		pos.x = next_x as i16;
		pos.y = next_y as u16;
		Some(pos)
	}
}

impl ChunkSubPos {
	pub fn new(x: u8, y: u8) -> ChunkSubPos {
		if x >= CHUNK_SIZE as u8 {
			panic!("X {} is bigger than {} which is the chunk size.", x, CHUNK_SIZE);
		}
		if y >= CHUNK_SIZE as u8 {
			panic!("Y {} is bigger than {} which is the chunk size.", y, CHUNK_SIZE);
		}
		Self {
			x,
			y,
		}
	}

	pub fn new_overlooping(mut x: i8, mut y: i8) -> ChunkSubPos {
		if x >= CHUNK_SIZE as i8 {
			x = 0;
		}
		if y >= CHUNK_SIZE as i8 {
			y = 0;
		}
		if x < 0 as i8 {
			x = (CHUNK_SIZE - 1) as i8;
		}
		if y < 0 as i8 {
			y = (CHUNK_SIZE - 1) as i8;
		}

		Self {
			x: x as u8,
			y: y as u8,
		}
	}

	pub fn is_border(&self, direction: &Direction) -> bool {
		match direction {
			Direction::Down => self.y <= 0,
			Direction::Left => self.x <= 0,
			Direction::Top => self.y >= (CHUNK_SIZE - 1) as u8,
			Direction::Right => self.x >= (CHUNK_SIZE - 1) as u8,
		}
	}

	pub fn shift_overlooping(&self, direction: Direction) -> ChunkSubPos {
		Self::new_overlooping((self.x as i32 + direction.get_x_difference() as i32) as i8,
							  (self.y as i32 + direction.get_y_difference() as i32) as i8)
	}

	pub fn shift(&self, direction: Direction) -> Option<ChunkSubPos> {
		let next_x = (self.x as i32 + direction.get_x_difference() as i32);
		let next_y = (self.y as i32 + direction.get_y_difference() as i32);
		if next_y < 0 || next_y > CHUNK_SIZE as i32 || next_x < 0 || next_x > CHUNK_SIZE as i32 {
			return None;
		}
		let mut pos = self.clone();
		pos.x = next_x as u8;
		pos.y = next_y as u8;
		Some(pos)
	}
}
