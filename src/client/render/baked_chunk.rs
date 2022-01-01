use glam::{Vec2, Vec3};
use std::ops::{Add, Sub};
use crate::client::render::world_renderer::NeighborImageLocation;
use crate::client::opengl::builder::{QuadBuilder, QuadDepthBuilder};
use crate::client::opengl::gl;
use crate::client::opengl::gl::{BufferUsage, VertexDivisor};
use crate::client::opengl::hlgl::{Atlas, ImageId};
use crate::client::opengl::sgl::{Program, VertexData};
use crate::client::viewport::Viewport;
use crate::misc::pos::ChunkPos;
use crate::misc::util::CHUNK_SIZE;
use crate::World;
use crate::world::tile::Tile;
use crate::world::{Grid, tile, wall};
use crate::world::wall::Wall;

pub struct BakedChunk {
	layout: VertexData,
	vertices: u32,
}

impl BakedChunk {
	pub fn new(
		program: &Program,
		viewport: &Viewport,
		world: &World,
		pos: &ChunkPos,
		tile_atlas: &Atlas,
	) -> Option<BakedChunk> {
		world.get_chunk(pos).map(|chunk| {
			let mut builder = ChunkVertexBuilder::new(viewport, pos);
			let mut vertices = 0u32;

			//for y in 0..CHUNK_SIZE {
			//	let walls_y: &[Wall; CHUNK_SIZE] = &chunk.get_grid()[y];
			//	for x in 0..CHUNK_SIZE {
			//		let wall_x = &walls_y[x];
//
//
			//		let i = Self::get_variant(pos, y, x);
			//		if wall_x.id != wall::AIR {
			//			vertices += 6; // quad
			//			builder.add_wall(x, y, wall_x, tile_atlas, i);
			//		}
			//	}
			//}

			for y in 0..CHUNK_SIZE {
				let tiles_y: &[Tile; CHUNK_SIZE] = &chunk.get_grid()[y];
				for x in 0..CHUNK_SIZE {
					let tile_x = &tiles_y[x];

					let i = Self::get_variant(pos, y, x);
					if tile_x.id != tile::AIR {
						vertices += 6; // quad
						builder.add_tile(x, y, tile_x, tile_atlas, i);
					}
				}
			}

			Self {
				layout: builder.export(program),
				vertices,
			}
		})
	}

	fn get_variant(pos: &ChunkPos, y: usize, x: usize) -> u64 {
		pub fn next(mut x: u64) -> u64 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			x
		}

		let i = next(
			next((x as u64) * 5)
				+ next(y as u64)
				+ next(pos.x.abs() as u64 * CHUNK_SIZE as u64)
				+ next(pos.y as u64 * CHUNK_SIZE as u64),
		);
		i
	}

	pub fn draw(&self, draw_type: u32) {
		self.layout.bind();
		gl::draw_arrays(draw_type, 0, self.vertices as i32);
		self.layout.unbind();
	}
}


pub struct ChunkVertexBuilder {
	pos: Vec<Vec3>,
	textures: Vec<Vec2>,
	gl_chunk_x: f32,
	gl_chunk_y: f32,
	gl_tile_width: f32,
	gl_tile_height: f32,
}

impl ChunkVertexBuilder {
	pub fn new(viewport: &Viewport, chunk_pos: &ChunkPos) -> ChunkVertexBuilder {
		Self {
			pos: Vec::new(),
			gl_chunk_x: (chunk_pos.x as f32) * (viewport.gl_tile_width * CHUNK_SIZE as f32),
			gl_chunk_y: (chunk_pos.y as f32) * (viewport.gl_tile_height * CHUNK_SIZE as f32),
			gl_tile_width: viewport.gl_tile_width,
			gl_tile_height: viewport.gl_tile_height,
			textures: Vec::new()
		}
	}

	pub fn add_tile(&mut self, x: usize, y: usize, tile: &Tile, atlas: &Atlas, var: u64) {
		let gl_pos = Vec2::new(
			self.gl_chunk_x + (x as f32 * self.gl_tile_width),
			self.gl_chunk_y + ((y as f32 + 1f32) * self.gl_tile_height),
		);

		// Tile type position of the sprite.
		let (type_x, type_y) = NeighborImageLocation::from(tile).get_tile_pos();
		let image = atlas.get_image(ImageId::Tile(tile.get_id().clone()));
		let image_pos = Vec2::new(image.x, image.y);

		// We have 3 variants. An entry of tiles is 5x4 but are stacked horizontally for every variant.
		let variant_offset = (var % 3u64) as f32 * 4f32;

		// Out layout is 12 x 5.
		// A single tile is always (image.width / 12f32, image.height / 5f32) of size.
		let item_tile_size = Vec2::new(
			image.width / 12f32,
			image.height / 5f32,
		);

		// ________
		// _      _
		// _      _
		// ________
		// ^ gl_pos

		// Calculate the offset on the tile sprite.
		let image_offset = Vec2::new(
			(type_x as f32 + variant_offset) * item_tile_size.x,
			(type_y as f32) * item_tile_size.y,
		);

		// Add stuff
		self.pos.add_quad(gl_pos, Vec2::new(self.gl_tile_width, -self.gl_tile_height), 1f32);
		self.textures.add_quad(image_pos.add(image_offset), item_tile_size);
	}

	pub fn export(self, program: &Program) -> VertexData {
		let mut layout = VertexData::new(2);
		layout.add_vertex_array(&program.get_attribute("in_Position"), self.pos, BufferUsage::StaticDraw, VertexDivisor::Vertex);
		layout.add_vertex_array(&program.get_attribute("in_TextureCoord"), self.textures, BufferUsage::StaticDraw, VertexDivisor::Vertex);
		layout
	}
}
