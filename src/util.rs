use std::ops::Deref;
use std::slice::Iter;
use crate::world::tile::{Tile};

pub const CHUNK_SIZE: usize = 24;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum CallbackResponse {
	Continue,
	Stop,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum CollisionType {
	CollidesPlayer,
	Nothing,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
	Top,
	Down,
	Left,
	Right,
}

impl Deref for Direction {
	type Target = ();

	fn deref(&self) -> &Self::Target {
		&self
	}
}


impl Direction {
	pub fn flip(&self) -> Direction {
		match self {
			Direction::Top => Direction::Down,
			Direction::Left => Direction::Right,
			Direction::Down => Direction::Top,
			Direction::Right => Direction::Left,
		}
	}

	pub fn clockwise(&self) -> Direction {
		match self {
			Direction::Top => Direction::Right,
			Direction::Right => Direction::Down,
			Direction::Down => Direction::Left,
			Direction::Left => Direction::Top,
		}
	}

	pub fn counter_clockwise(&self) -> Direction {
		match self {
			Direction::Top => Direction::Left,
			Direction::Left => Direction::Down,
			Direction::Down => Direction::Right,
			Direction::Right => Direction::Top,
		}
	}

	pub fn get_x_difference(&self) -> i8 {
		match self {
			Direction::Top | Direction::Down => 0,
			Direction::Left => -1,
			Direction::Right => 1,
		}
	}

	pub fn get_y_difference(&self) -> i8 {
		match self {
			Direction::Left | Direction::Right => 0,
			Direction::Top => 1,
			Direction::Down => -1,
		}
	}
	pub fn iter() -> Iter<'static, Direction> {
		static DIRECTIONS: [Direction; 4] = [Direction::Top, Direction::Down, Direction::Left, Direction::Right];
		DIRECTIONS.iter()
	}
}