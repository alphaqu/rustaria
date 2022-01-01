use std::ops::{Add};

use glam::{IVec2, UVec2, Vec2};
use glam::{IVec3, UVec3, Vec3};
use glam::{IVec4, UVec4, Vec4};

use crate::client::opengl::gl::DataType;



//impl VertexBuilderTrait<Vec3A> for Vec<Vec3A> {
//	fn get_data_type() -> (DataType, i32) {
//		(DataType::Float, 423)
//	}
//
//	fn add_quad(&mut self, x: Vec3A, y: Vec3A, width: Vec3A, height: Vec3A) {
//		self.push(x);
//		self.push(y);

//		self.push(x);
//		self.push(y.add(height));

//		self.push(x.add(width));
//		self.push(y);

//		self.push(x.add(width));
//		self.push(y);

//		self.push(x);
//		self.push(y.add(height));

//		self.push(x.add(width));
//		self.push(y.add(height));
//	}
//}

pub trait VertexBuilderTrait<E> {
	fn get_data_type() -> (DataType, i32);
}

pub trait QuadBuilder<E> {
	fn add_quad(&mut self, pos: E, dimensions: E);
}


impl QuadBuilder<Vec2> for Vec<Vec2> {
	fn add_quad(&mut self, pos: Vec2, size: Vec2) {
		// Order matters big brain. - small brain alpha
		self.push(pos.add(Vec2::new(0f32, size.y)));
		self.push(pos);
		self.push(pos.add(size));
		self.push(pos.add(size));
		self.push(pos);
		self.push(pos.add(Vec2::new(size.x, 0f32)));
		//let thing = Vec2::new(pos.x + size.x, pos.y - size.y);
	 	//self.push(thing);
	}
}

pub trait QuadDepthBuilder<E> {
	fn add_quad(&mut self, pos: Vec2, dimensions: Vec2, depth: f32);
}

impl QuadDepthBuilder<Vec3> for Vec<Vec3> {
	fn add_quad(&mut self, pos: Vec2, size: Vec2, depth: f32) {
		// Order matters big brain. - small brain alpha
		let point = pos.add(Vec2::new(0f32, size.y));
		self.push(Vec3::new(point.x, point.y, depth));
		self.push(Vec3::new(pos.x, pos.y, depth));
		let point = pos.add(size);
		self.push(Vec3::new(point.x, point.y, depth));
		self.push(Vec3::new(point.x, point.y, depth));
		self.push(Vec3::new(pos.x, pos.y, depth));
		let point = pos.add(Vec2::new(size.x, 0f32));
		self.push(Vec3::new(point.x, point.y, depth));

		//let thing = Vec2::new(pos.x + size.x, pos.y - size.y);
		//self.push(thing);
	}
}

macro_rules! vertex_support {
    ($($TYPE:ty => ($DATA_TYPE:expr, $AMOUNT:literal);)*) => {
		$(
			impl VertexBuilderTrait<$TYPE> for Vec<$TYPE> {
				fn get_data_type() -> (DataType, i32) {
					($DATA_TYPE, $AMOUNT)
				}
			}
		)*
	};
}

vertex_support!(
	f32 => (DataType::Float, 1);
	i32 => (DataType::IInt, 1);
	u32 => (DataType::UInt, 1);
	Vec2 => (DataType::Float, 2);
	IVec2 => (DataType::IInt, 2);
	UVec2 => (DataType::UInt, 2);
	Vec3 => (DataType::Float, 3);
	IVec3 => (DataType::IInt, 3);
	UVec3 => (DataType::UInt, 3);
	Vec4 => (DataType::Float, 4);
	IVec4 => (DataType::IInt, 4);
	UVec4 => (DataType::UInt, 4);
);
