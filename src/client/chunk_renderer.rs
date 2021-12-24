use std::collections::HashMap;
use std::fs::read_dir;
use std::ops::{Add, Sub};
use std::path::Path;

use glam::{Vec2, Vec3};
use image::GenericImageView;

use crate::{Player, read_asset_string, Settings};
use crate::client::opengl::builder::{QuadBuilder, QuadDepthBuilder};
use crate::client::opengl::gl;
use crate::client::opengl::gl::{BufferUsage, DataType, VertexDivisor};
use crate::client::opengl::hlgl::{Atlas, AtlasGroup, AtlasSettings, Image, ImageId, Program, Uniform, VertexLayout};
use crate::client::viewport::Viewport;
use crate::pos::{ChunkPos, WorldPos};
use crate::util::{CHUNK_SIZE, Direction};
use crate::world::{Grid, tile, wall, World};
use crate::world::neighbor::NeighborAware;
use crate::world::tile::Tile;
use crate::world::wall::Wall;

pub struct ChunkRenderer {
	debug: bool,
	program: Program,
	player_pos: Uniform,
	gl_zoom: Uniform,

	texture_sampler: Uniform,
	atlas: Atlas,
	baked_chunks: HashMap<ChunkPos, BakedChunk>,
}

impl ChunkRenderer {
	pub fn new(_player: &Player) -> ChunkRenderer {
		let program = Program::new(
			read_asset_string("shader/tile-fragment.glsl"),
			read_asset_string("shader/tile-vertex.glsl"),
		);


		println!("Reading images");
		let mut images = Vec::new();
		for entry in read_dir(Path::new("C:\\Program Files (x86)\\inkscape\\cppProjects\\rustaria\\assets\\sprite\\tile")).unwrap() {
			let dir = entry.unwrap();
			let buf = dir.path();

			let image = Image::load(buf.as_ref());
			images.push(image);
			println!("Added {:?}", buf);
		}

		for entry in read_dir(Path::new("C:\\Program Files (x86)\\inkscape\\cppProjects\\rustaria\\assets\\sprite\\wall")).unwrap() {
			let dir = entry.unwrap();
			let buf = dir.path();

			let image = Image::load(buf.as_ref());
			images.push(image);
			println!("Added {:?}", buf);
		}

		let atlas = Atlas::new(images, AtlasSettings {
			mipmaps: 3
		});

		Self {
			debug: false,
			player_pos: Uniform::new(&program, "glPlayerPos"),
			gl_zoom: Uniform::new(&program, "glZoom"),
			texture_sampler: Uniform::new(&program, "texture_sampler"),
			atlas: atlas,
			program,
			baked_chunks: HashMap::new(),
		}
	}

	pub fn debug_mode(&mut self) {
		self.debug = !self.debug;
	}

	pub fn tick(
		&mut self,
		viewport: &Viewport,
		player: &Player,
		world: &mut World,
		settings: &Settings,
	) {
		let pos = ChunkPos::from_player(player);
		self.process_chunks(viewport, world, &pos, settings.render_distance);
	}

	fn process_chunks(
		&mut self,
		viewport: &Viewport,
		world: &mut World,
		player_pos: &ChunkPos,
		render_distance: u16,
	) {
		let render_distance_half = (render_distance as i32 / 2i32) as i32;

		let mut lookup_pos = player_pos.clone();
		for y in 0..(render_distance + 1) {
			let chunk_y: i32 = player_pos.y as i32 + (y as i32 - render_distance_half);
			if chunk_y < 0 {
				continue;
			}
			lookup_pos.y = chunk_y as u16;

			for x in 0..(render_distance + 1) {
				lookup_pos.x = (player_pos.x as i32 + (x as i32 - render_distance_half)) as i16;
				if !self.baked_chunks.contains_key(&lookup_pos) {
					let pos1 = &lookup_pos;
					world.poll_chunk(pos1);
					BakedChunk::new(viewport, world, pos1, &self.atlas).map(|baked_chunk| {
						self.baked_chunks.insert(lookup_pos.clone(), baked_chunk);
					});
				}
			}
		}

		self.baked_chunks.retain(|pos, _baked_chunk| {
			(player_pos.y as i32 - pos.y as i32).abs() < (render_distance as i32)
				&& (player_pos.x as i32 - pos.x as i32).abs() < (render_distance as i32)
		})
	}

	pub fn tile_change(&mut self, pos: &WorldPos) {
		let chunk_tile_pos = pos.get_chunk_sub_pos();
		for dir in Direction::iter() {
			if chunk_tile_pos.is_border(dir) {
				pos.get_chunk_pos().shift(dir).map(|pos| {
					self.baked_chunks.remove(&pos);
				});
			}
		}
		self.baked_chunks.remove(pos.get_chunk_pos());
	}

	pub fn rebuild_all(&mut self) {
		self.baked_chunks.clear();
	}

	pub fn rebuild_chunk(&mut self, chunk_pos: &ChunkPos) {
		self.baked_chunks.remove(chunk_pos);
	}

	pub fn draw(&self, viewport: &Viewport, player: &Player, settings: &Settings) {
		self.program.bind();
		self.player_pos.uniform_2f(
			player.pos_x * viewport.gl_tile_width,
			player.pos_y * viewport.gl_tile_height,
		);
		self.gl_zoom.uniform_1f(settings.zoom);
		self.texture_sampler.uniform_1i(0);
		self.atlas.bind();

		let draw_mode = if self.debug { gl::LINE_STRIP } else { gl::TRIANGLES };

		if settings.cull_chunks {
			let player_chunk = ChunkPos::from_player(player);
			let chunks_x = (1f32 / (viewport.gl_tile_width * CHUNK_SIZE as f32)) * settings.zoom;
			let chunks_y = (1f32 / (viewport.gl_tile_height * CHUNK_SIZE as f32)) * settings.zoom;

			for (pos, baked_chunk) in &self.baked_chunks {
				if ((pos.x as f32 - player_chunk.x as f32).abs() > (chunks_x + 1f32))
					|| ((pos.y as f32 - player_chunk.y as f32).abs() > (chunks_y + 1f32))
				{
					continue;
				}

				baked_chunk.draw(draw_mode);
			}
		} else {
			for (_pos, baked_chunk) in &self.baked_chunks {
				baked_chunk.draw(draw_mode);
			}
		}
		self.program.unbind();
	}
}

pub struct BakedChunk {
	layout: VertexLayout,
	vertices: u32,
}

impl BakedChunk {
	pub fn new(
		viewport: &Viewport,
		world: &World,
		pos: &ChunkPos,
		tile_atlas: &Atlas,
	) -> Option<BakedChunk> {
		world.get_chunk(pos).map(|chunk| {
			let mut builder = ChunkVertexBuilder::new(viewport, pos);
			let mut vertices = 0u32;

			for y in 0..CHUNK_SIZE {
				let walls_y: &[Wall; CHUNK_SIZE] = &chunk.get_grid()[y];
				for x in 0..CHUNK_SIZE {
					let wall_x = &walls_y[x];


					let i = Self::get_variant(pos, y, x);
					if wall_x.id != wall::AIR {
						vertices += 6; // quad
						builder.add_wall(x, y, wall_x, tile_atlas, i);
					}
				}
			}

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
				layout: builder.export(),
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
			textures: Vec::new(),
			gl_chunk_x: (chunk_pos.x as f32) * (viewport.gl_tile_width * CHUNK_SIZE as f32),
			gl_chunk_y: (chunk_pos.y as f32) * (viewport.gl_tile_height * CHUNK_SIZE as f32),
			gl_tile_width: viewport.gl_tile_width,
			gl_tile_height: viewport.gl_tile_height,
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

	pub fn add_wall(&mut self, x: usize, y: usize, wall: &Wall, atlas: &Atlas, var: u64) {
		// Tile type position of the sprite.
		let ((w_x, w_y), (width, height)) = NeighborImageLocation::from(wall).get_wall_pos();
		let image = atlas.get_image(ImageId::Wall(wall.get_id().clone()));
		let image_pos = Vec2::new(image.x, image.y);

		// - 0.5 is because of the wall texture
		let gl_pos = Vec2::new(
			self.gl_chunk_x + (((x as f32 - 0.5) + w_x) * self.gl_tile_width),
			self.gl_chunk_y + (((y as f32 - 0.5) + w_y) * self.gl_tile_height),
		);

		// We have 3 variants. An entry of tiles is 2x2 but are stacked horizontally for every variant.
		let variant_offset = (var % 3u64) as f32 * 2f32;

		let item_tile_size = Vec2::new(
			image.width / 6f32,
			image.height / 2f32,
		);

		// Calculate the offset on the tile sprite.
		let image_offset = Vec2::new(
			(w_x as f32 + variant_offset) * item_tile_size.x,
			(w_y as f32) * item_tile_size.y,
		);

		let image_size = Vec2::new(
			item_tile_size.x * width,
			item_tile_size.y * height,
		);

		// Add stuff
		let y1 = (height) * self.gl_tile_height;
		self.pos.add_quad(gl_pos.sub(Vec2::new(0f32, y1)), Vec2::new(width * self.gl_tile_width, -y1), 0f32);
		self.textures.add_quad(image_pos.add(image_offset), image_size);
	}

	pub fn export(self) -> VertexLayout {
		let mut layout = VertexLayout::new(2);
		layout.add_vbo(0, self.pos,      BufferUsage::StaticDraw, VertexDivisor::Vertex);
		layout.add_vbo(1, self.textures, BufferUsage::StaticDraw, VertexDivisor::Vertex);
		layout
	}
}

pub enum NeighborImageLocation {
	Full,
	Standalone,
	StraightVertical,
	StraightHorizontal,
	TopFlat,
	TopCap,
	TopLeftCorner,
	DownFlat,
	DownCap,
	DownLeftCorner,
	LeftFlat,
	LeftCap,
	TopRightCorner,
	RightFlat,
	RightCap,
	DownRightCorner,
}

impl NeighborImageLocation {
	pub fn from<N: NeighborAware>(object: &N) -> NeighborImageLocation {
		use crate::world::neighbor::NeighborType::{Air, Same};
		let matrix = object.get_neighbor_matrix();
		match (
			matrix.get_neighbor_type(Direction::Top),
			matrix.get_neighbor_type(Direction::Down),
			matrix.get_neighbor_type(Direction::Left),
			matrix.get_neighbor_type(Direction::Right),
		) {
			(Same, Same, Same, Same) => NeighborImageLocation::Full,
			(Air, Air, Air, Air) => NeighborImageLocation::Standalone,
			(Same, Same, Air, Air) => NeighborImageLocation::StraightVertical,
			(Air, Air, Same, Same) => NeighborImageLocation::StraightHorizontal,

			(Air, Same, Same, Same) => NeighborImageLocation::TopFlat,
			(Air, Same, Air, Air) => NeighborImageLocation::TopCap,
			(Air, Same, Air, Same) => NeighborImageLocation::TopLeftCorner,
			(Air, Same, Same, Air) => NeighborImageLocation::TopRightCorner,

			(Same, Air, Same, Same) => NeighborImageLocation::DownFlat,
			(Same, Air, Air, Air) => NeighborImageLocation::DownCap,
			(Same, Air, Air, Same) => NeighborImageLocation::DownLeftCorner,
			(Same, Air, Same, Air) => NeighborImageLocation::DownRightCorner,

			(Same, Same, Air, Same) => NeighborImageLocation::LeftFlat,
			(Air, Air, Air, Same) => NeighborImageLocation::LeftCap,
			(Same, Same, Same, Air) => NeighborImageLocation::RightFlat,
			(Air, Air, Same, Air) => NeighborImageLocation::RightCap,
			_ => NeighborImageLocation::Full,
		}
	}

	pub fn get_tile_pos(&self) -> (u32, u32) {
		match self {
			NeighborImageLocation::Full => (0, 0),
			NeighborImageLocation::Standalone => (1, 0),
			NeighborImageLocation::StraightVertical => (2, 0),
			NeighborImageLocation::StraightHorizontal => (3, 0),
			NeighborImageLocation::TopFlat => (0, 1),
			NeighborImageLocation::TopCap => (1, 1),
			//NeighborImageLocation::TopReach => (2, 1),
			NeighborImageLocation::TopLeftCorner => (3, 1),
			NeighborImageLocation::DownFlat => (0, 2),
			NeighborImageLocation::DownCap => (1, 2),
			//NeighborImageLocation::DownReach => (2, 2),
			NeighborImageLocation::DownLeftCorner => (3, 2),
			NeighborImageLocation::LeftFlat => (0, 3),
			NeighborImageLocation::LeftCap => (1, 3),
			//NeighborImageLocation::LeftReach => (2, 3),
			NeighborImageLocation::TopRightCorner => (3, 3),
			NeighborImageLocation::RightFlat => (0, 4),
			NeighborImageLocation::RightCap => (1, 4),
			//NeighborImageLocation::RightReach => (2, 4),
			NeighborImageLocation::DownRightCorner => (3, 4),
		}
	}
	pub fn get_wall_pos(&self) -> ((f32, f32), (f32, f32)) {
		match self {
			NeighborImageLocation::Full => ((0.5, 0.5), (1.0, 1.0)),
			NeighborImageLocation::Standalone => ((0.0, 0.0), (2.0, 2.0)),
			NeighborImageLocation::StraightVertical => ((0.0, 0.5), (2.0, 1.0)),
			NeighborImageLocation::StraightHorizontal => ((0.5, 0.0), (1.0, 2.0)),

			NeighborImageLocation::TopFlat => ((0.5, 0.0), (1.0, 1.5)),
			NeighborImageLocation::TopCap => ((0.0, 0.0), (2.0, 1.5)),
			NeighborImageLocation::TopLeftCorner => ((0.0, 0.0), (1.5, 1.5)),

			NeighborImageLocation::DownFlat => ((0.5, 0.5), (1.0, 1.5)),
			NeighborImageLocation::DownCap => ((0.0, 0.5), (2.0, 1.5)),
			NeighborImageLocation::DownLeftCorner => ((0.0, 0.5), (1.5, 1.5)),

			NeighborImageLocation::LeftFlat => ((0.0, 0.5), (1.5, 1.0)),
			NeighborImageLocation::LeftCap => ((0.0, 0.0), (1.5, 2.0)),
			NeighborImageLocation::TopRightCorner => ((0.5, 0.0), (1.5, 1.5)),

			NeighborImageLocation::RightFlat => ((0.5, 0.5), (1.5, 1.0)),
			NeighborImageLocation::RightCap => ((0.5, 0.0), (1.5, 2.0)),
			NeighborImageLocation::DownRightCorner => ((0.5, 0.5), (1.5, 1.5)),
		}
	}
}

