use std::ops::Deref;
use crate::misc::util::Direction;
use crate::world::tile::Tile;

pub trait NeighborAware {
	fn get_neighbor_matrix(&self) -> &NeighborMatrix;

	fn apply_neighbor(&self, neighbor: &Self) -> NeighborType;
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct NeighborMatrix {
	top: NeighborType,
	down: NeighborType,
	left: NeighborType,
	right: NeighborType,
}

impl NeighborMatrix {
	pub fn new() -> Self {
		Self {
			top: NeighborType::Air,
			down: NeighborType::Air,
			left: NeighborType::Air,
			right: NeighborType::Air,
		}
	}

	pub fn set_neighbor_type(&mut self, direction: Direction, neighbor_type: NeighborType) {
		match direction {
			Direction::Top => self.top = neighbor_type,
			Direction::Down => self.down = neighbor_type,
			Direction::Left => self.left = neighbor_type,
			Direction::Right => self.right = neighbor_type,
		}
	}

	pub fn get_neighbor_type(&self, direction: Direction) -> NeighborType {
		match direction {
			Direction::Top => self.top.clone(),
			Direction::Down => self.down.clone(),
			Direction::Left => self.left.clone(),
			Direction::Right => self.right.clone(),
		}
	}

	/// Actually mutates the values. watch out!
	pub unsafe fn update_neighbor<C: NeighborAware>(source: &C, neighbor: &C, direction: Direction) {

		unsafe {
			let holder_ptr = source.get_neighbor_matrix() as *const NeighborMatrix as *mut NeighborMatrix;
			let neighbor_ptr = neighbor.get_neighbor_matrix() as *const NeighborMatrix as *mut NeighborMatrix;

			(holder_ptr.as_mut().unwrap()).set_neighbor_type(direction, source.apply_neighbor(neighbor));
			(neighbor_ptr.as_mut().unwrap()).set_neighbor_type(direction.flip(), neighbor.apply_neighbor(source));
		}
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum NeighborType {
	Air,
	Same,
	Transitional,
}
