#![allow(dead_code)]
#![allow(unused_variables)]

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::iter::Map;
use std::path::Path;

use image::{ColorType, DynamicImage, GenericImage, GenericImageView, ImageBuffer};
use image::imageops::FilterType;
use potpack::{Id, PotPack, SizedItem};
use rectangle_pack::{contains_smallest_box, GroupedRectsToPlace, pack_rects, RectanglePackError, RectanglePackOk, RectToInsert, TargetBin, volume_heuristic};

use crate::client::opengl::builder::VertexBuilderTrait;
// use crate::client::opengl::gl;
use crate::client::opengl::gl::{
	self, BufferTarget, BufferType, BufferUsage, DataType, VertexDivisor,
};
use crate::consts::{TileId, WallId};

use super::gll::types::{GLenum, GLuint};

pub struct Program {
	program_id: GLuint,
	fragment_id: GLuint,
	vertex_id: GLuint,
}

impl Program {
	pub fn new(fragment: String, vertex: String) -> Program {
		let program_id = gl::create_program();
		let fragment_id = Self::create_shader(program_id, gl::FRAGMENT_SHADER, fragment);
		let vertex_id = Self::create_shader(program_id, gl::VERTEX_SHADER, vertex);

		gl::link_program(program_id);

		if vertex_id != 0 {
			gl::detach_shader(program_id, vertex_id);
		}
		if fragment_id != 0 {
			gl::detach_shader(program_id, fragment_id);
		}

		gl::validate_program(program_id);

		Self {
			program_id,
			fragment_id,
			vertex_id,
		}
	}

	pub fn create_shader(program_id: GLuint, shader: GLenum, code: String) -> GLuint {
		let shader_id = gl::create_shader(shader);
		gl::shader_source(shader_id, code);
		gl::compile_shader(shader_id, program_id);
		gl::attach_shader(program_id, shader_id);
		shader_id
	}

	pub fn bind(&self) {
		gl::attach_shader(self.program_id, self.vertex_id);
		gl::attach_shader(self.program_id, self.fragment_id);
		gl::use_program(self.program_id);
	}

	pub fn unbind(&self) {
		gl::detach_shader(self.program_id, self.vertex_id);
		gl::detach_shader(self.program_id, self.fragment_id);
		gl::use_program(0);
	}
}

impl Drop for Program {
	fn drop(&mut self) {
		if self.program_id != 0 {
			gl::delete_program(self.program_id);
		}
	}
}

pub struct Uniform {
	location: i32,
}

impl Uniform {
	pub fn new(program: &Program, name: &str) -> Uniform {
		let location = gl::get_uniform_location(program.program_id, name);
		Self { location }
	}

	pub fn uniform_1d(&self, v0: f64) {
		gl::uniform_1d(self.location, v0);
	}

	pub fn uniform_1f(&self, v0: f32) {
		gl::uniform_1f(self.location, v0);
	}

	pub fn uniform_1i(&self, v0: i32) {
		gl::uniform_1i(self.location, v0);
	}

	pub fn uniform_1ui(&self, v0: u32) {
		gl::uniform_1ui(self.location, v0);
	}

	pub fn uniform_2d(&self, v0: f64, v1: f64) {
		gl::uniform_2d(self.location, v0, v1);
	}

	pub fn uniform_2f(&self, v0: f32, v1: f32) {
		gl::uniform_2f(self.location, v0, v1);
	}

	pub fn uniform_2i(&self, v0: i32, v1: i32) {
		gl::uniform_2i(self.location, v0, v1);
	}

	pub fn uniform_2ui(&self, v0: u32, v1: u32) {
		gl::uniform_2ui(self.location, v0, v1);
	}

	pub fn uniform_3d(&self, v0: f64, v1: f64, v2: f64) {
		gl::uniform_3d(self.location, v0, v1, v2);
	}

	pub fn uniform_3f(&self, v0: f32, v1: f32, v2: f32) {
		gl::uniform_3f(self.location, v0, v1, v2);
	}

	pub fn uniform_3i(&self, v0: i32, v1: i32, v2: i32) {
		gl::uniform_3i(self.location, v0, v1, v2);
	}

	pub fn uniform_3ui(&self, v0: u32, v1: u32, v2: u32) {
		gl::uniform_3ui(self.location, v0, v1, v2);
	}

	pub fn uniform_4d(&self, v0: f64, v1: f64, v2: f64, v3: f64) {
		gl::uniform_4d(self.location, v0, v1, v2, v3);
	}

	pub fn uniform_4f(&self, v0: f32, v1: f32, v2: f32, v3: f32) {
		gl::uniform_4f(self.location, v0, v1, v2, v3);
	}

	pub fn uniform_4i(&self, v0: i32, v1: i32, v2: i32, v3: i32) {
		gl::uniform_4i(self.location, v0, v1, v2, v3);
	}

	pub fn uniform_4ui(&self, v0: u32, v1: u32, v2: u32, v3: u32) {
		gl::uniform_4ui(self.location, v0, v1, v2, v3);
	}
}

// Vertex Array Objects (VAOs)
pub struct VertexAttribute {
	index: u32,
	amount: i32,
	data_type: DataType,
}

impl VertexAttribute {
	pub fn new(index: u32, amount: i32, data_type: DataType) -> VertexAttribute {
		Self {
			index,
			amount,
			data_type,
		}
	}
}

pub struct VertexLayout {
	id: GLuint,
	vertex_buffers: Vec<VertexBuffer>,
}

impl VertexLayout {
	pub fn new(buffers: usize) -> VertexLayout {
		let id = gl::gen_vertex_arrays();

		Self {
			id,
			vertex_buffers: Vec::with_capacity(buffers),
		}
	}

	pub fn add_vbo<E>(
		&mut self,
		index: u32,
		data: Vec<E>,
		usage: BufferUsage,
		instance: VertexDivisor,
	) where Vec<E>: VertexBuilderTrait<E> {
		gl::bind_vertex_array(&self.id);
		let vbo = VertexBuffer::new(&data, usage);
		gl::vertex_attrib_pointer(index, vbo.amount, &vbo.data_type, false, vbo.element_size as i32);
		gl::vertex_attrib_divisor(index, instance);

		vbo.unbind();
		gl::bind_vertex_array(&0);

		self.vertex_buffers.insert(index as usize, vbo);
	}

	pub fn bind(&self) {
		gl::bind_vertex_array(&self.id);
		for i in 0..(self.vertex_buffers.len()) {
			gl::enable_vertex_attribute_array(i as u32);
		}
	}

	pub fn unbind(&self) {
		for i in 0..(self.vertex_buffers.len()) {
			gl::disable_vertex_attribute_array(i as u32);
		}
		gl::bind_vertex_array(&0);
	}
}

impl Drop for VertexLayout {
	fn drop(&mut self) {
		self.unbind();
		gl::delete_vertex_array(self.id);
	}
}

pub struct VertexBuffer {
	id: GLuint,
	amount: i32,
	element_size: usize,
	data_type: DataType,
	buffer_type: BufferType,
}

impl VertexBuffer {
	pub fn new<E>(
		data: &Vec<E>,
		usage: BufferUsage,
	) -> VertexBuffer where Vec<E>: VertexBuilderTrait<E> {
		let id = gl::gen_buffer();
		gl::bind_buffer(&BufferType::ArrayBuffer, &id);
		gl::buffer_data_vec(BufferTarget::ArrayBuffer, &data, usage);

		let data_type = Vec::<E>::get_data_type();
		Self {
			id,
			amount: data_type.1,
			element_size: std::mem::size_of::<E>() as usize,
			data_type: data_type.0,
			buffer_type: BufferType::ArrayBuffer,
		}
	}

	pub fn bind(&self) {
		gl::bind_buffer(&self.buffer_type, &self.id);
	}

	pub fn unbind(&self) {
		gl::bind_buffer(&self.buffer_type, &0);
	}
}

impl Drop for VertexBuffer {
	fn drop(&mut self) {
		self.unbind();
		gl::delete_buffer(self.id);
	}
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd)]
pub enum AtlasGroup {
	Tiles,
}

pub struct AtlasSettings {
	pub mipmaps: u32,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd)]
pub enum ImageId {
	Tile(TileId),
	Wall(WallId),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Image {
	pub id: ImageId,
	pub image: DynamicImage,
}

impl Image {
	pub fn new(image: DynamicImage, id: ImageId) -> Image {
		Self {
			id,
			image,
		}
	}

	pub fn load(path: &Path) -> Image {
		let image = image::open(path).expect(&*format!("Could not find image at {}", path.to_str().unwrap()));
		// TYPE-ID.png <- all images follow this format
		let parts: Vec<&str> = path.file_stem().unwrap().to_str().unwrap().split('-').collect();
		let id: u32 = parts[1].parse().expect("Could not parse id.");
		let possible_type = parts[0];
		let image_type = match possible_type {
			"tile" => ImageId::Tile(TileId { id }),
			"wall" => ImageId::Wall(WallId { id }),
			&_ => panic!("Could not identify image type called {}", possible_type)
		};

		Self {
			id: image_type,
			image,
		}
	}
}

pub struct Atlas {
	id: GLuint,
	images: HashMap<ImageId, AtlasImage>,
}

impl Atlas {
	pub fn new(images: Vec<Image>, settings: AtlasSettings) -> Atlas {

		// Pack all of the images
		let (placement, atlas_w, atlas_h) = Self::pack_images(&images);

		// Allocate Atlas on the atlas size.
		println!("Allocating {}x{} atlas.", atlas_w, atlas_h);
		let mipmaps = settings.mipmaps;
		let id = gl::gen_texture();
		gl::bind_texture(gl::TEXTURE_2D, id);
		gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAX_LOD, mipmaps);
		gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_LOD, 0);
		gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, mipmaps);
		gl::tex_parameter_f(gl::TEXTURE_2D, gl::TEXTURE_LOD_BIAS, 0.1f32);
		for i in 0..=mipmaps {
			gl::tex_image_2d(
				gl::TEXTURE_2D,
				i as i32,
				gl::RGBA,
				atlas_w >> i,
				atlas_h >> i,
				0,
				gl::RGBA,
				gl::UNSIGNED_BYTE,
				Option::None,
			);
		};

		// Apply images
		let mut image_lookup = HashMap::with_capacity(images.len());
		for (image_id, (_, rect)) in placement.packed_locations() {
			let image = &images[*image_id];
			for i in 0..=mipmaps {
				let pixels = Self::get_pixels(&image.image, i);
				gl::pixel_store_i(gl::UNPACK_ALIGNMENT, 1);
				gl::tex_parameter_i(
					gl::TEXTURE_2D,
					gl::TEXTURE_MIN_FILTER,
					gl::NEAREST_MIPMAP_LINEAR,
				);
				gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST);
				gl::tex_sub_image_2d(
					gl::TEXTURE_2D,
					i as i32,
					rect.x() as u32 >> i,
					rect.y() as u32 >> i,
					rect.width() as u32 >> i,
					rect.height() as u32 >> i,
					gl::RGBA,
					gl::UNSIGNED_BYTE,
					pixels.as_ptr(),
				);
			}


			image_lookup.insert(
				image.id.clone(),
				AtlasImage {
					x: Self::gl_pos(atlas_w, rect.x() as f32),
					y: Self::gl_pos(atlas_h, rect.y() as f32),
					width: Self::gl_pos(atlas_w, rect.width() as f32),
					height: Self::gl_pos(atlas_h, rect.height() as f32),
				},
			);
		}

		for i in 0..=mipmaps {
			println!("savign thigs {}", i);
			let pbo = gl::gen_buffer();
			gl::bind_buffer(&BufferType::PixelPackBuffer, &pbo);
			let width = (atlas_w >> i);
			let height = (atlas_h >> i);
			let size = (width * height * 4) as usize;
			gl::buffer_data::<u8>(BufferTarget::PixelPackBuffer, size, None, BufferUsage::StreamRead);
			gl::get_tex_image(gl::TEXTURE_2D, i as i32, gl::RGBA, width, height, 0, gl::UNSIGNED_BYTE);
			let mut out: Vec<u8> = Vec::with_capacity(size);
			for j in 0..size {
				out.push(0);
			}
			gl::get_buffer_subdata(gl::PIXEL_PACK_BUFFER, 0, size, &mut out);
			image::save_buffer(format!("C:\\Program Files (x86)\\inkscape\\cppProjects\\rustaria\\archive\\{}-tile-atlas.png", i), out.as_slice(), width, height, ColorType::Rgba8);
			gl::delete_buffer(pbo);
		}

		Self {
			id,
			images: image_lookup,
		}
	}

	pub fn get_image(&self, id: ImageId) -> &AtlasImage {
		self.images
			.get(&id)
			.expect(&*format!("Could not find image on tile id {:?}", id))
	}

	fn pack_images(images: &Vec<Image>) -> (RectanglePackOk<usize, i32>, u32, u32) {
		let image_amount = images.len();
		println!("Packing {} images.", image_amount);
		let mut rects_to_place = GroupedRectsToPlace::new();

		for id in 0..image_amount {
			let image = &images[id];
			rects_to_place.push_rect(
				id,
				Some(vec![69420u128]),
				RectToInsert::new(image.image.width(), image.image.height(), 1),
			);
		}


		let mut atlas_w = 256u32;
		let mut atlas_h = 256u32;
		loop {
			let mut target_bins = BTreeMap::new();
			target_bins.insert(1, TargetBin::new(atlas_w, atlas_h, 1));
			let rectangle_placements = match pack_rects(
				&rects_to_place,
				&mut target_bins,
				&volume_heuristic,
				&contains_smallest_box,
			) {
				Ok(placement) => {
					return (placement, atlas_w, atlas_h);
				}
				Err(err) => {
					match err {
						RectanglePackError::NotEnoughBinSpace => {
							if atlas_h > atlas_w {
								atlas_w = atlas_w << 1;
							} else {
								atlas_h = atlas_h << 1;
							}
							println!("Resized Atlas to {}x{}", atlas_w, atlas_h);
						}
					}
				}
			};
		};
	}

	fn get_pixels(image: &DynamicImage, level: u32) -> Vec<u8> {
		if level == 0 {
			image.to_bytes()
		} else {
			image.resize(image.width() >> level, image.height() >> level, FilterType::Nearest).to_bytes()
		}
	}

	pub fn bind(&self) {
		gl::active_texture(gl::TEXTURE0);
		gl::bind_texture(gl::TEXTURE_2D, self.id);
	}

	fn gl_pos(size: u32, pos: f32) -> f32 {
		pos as f32 / size as f32
	}
}


pub struct OldImage {
	pub width: u32,
	pub height: u32,
	pub data: DynamicImage,
}

impl OldImage {
	fn get_pixels(&self, level: u32) -> Vec<u8> {
		if level == 0 {
			self.data.to_bytes()
		} else {
			self.data
				.resize(
					self.data.width() >> level,
					self.data.height() >> level,
					FilterType::Nearest,
				)
				.to_bytes()
		}
	}
}

pub struct OldAtlas {
	id: GLuint,
	images: HashMap<u32, AtlasImage>,
}

impl OldAtlas {
	pub fn get_image(&self, id: &TileId) -> &AtlasImage {
		self.images
			.get(&id.id)
			.expect(&*format!("Could not find image on tile id {}", id.id))
	}

	pub fn create<'a>(
		data: HashMap<u32, OldImage>,
		atlas_width: u32,
		atlas_height: u32,
		group: AtlasGroup,
	) -> OldAtlas {
		println!("Stitching {} images", data.len());
		let mut rects_to_place = GroupedRectsToPlace::new();
		for (id, image) in &data {
			rects_to_place.push_rect(
				TileId { id: *id },
				Some(vec![group.clone()]),
				RectToInsert::new(image.width, image.height, 1),
			);
		}

		let mut target_bins = BTreeMap::new();
		target_bins.insert(1, TargetBin::new(atlas_width, atlas_height, 1));

		let rectangle_placements = pack_rects(
			&rects_to_place,
			&mut target_bins,
			&volume_heuristic,
			&contains_smallest_box,
		)
			.unwrap();

		println!("Allocating {} images", &data.len());
		let id = gl::gen_texture();

		let mipmaps = 4;

		gl::bind_texture(gl::TEXTURE_2D, id);
		gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAX_LOD, mipmaps - 1);
		gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_LOD, 0);
		gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, mipmaps - 1);
		gl::tex_parameter_f(gl::TEXTURE_2D, gl::TEXTURE_LOD_BIAS, 0.1f32);

		for i in 0..mipmaps {
			gl::tex_image_2d(
				gl::TEXTURE_2D,
				i as i32,
				gl::RGBA,
				atlas_width >> i,
				atlas_height >> i,
				0,
				gl::RGBA,
				gl::UNSIGNED_BYTE,
				Option::None,
			);
		}

		let mut images: HashMap<u32, AtlasImage> = HashMap::new();
		let locations = rectangle_placements.packed_locations();
		for (image_id, (bin_id, location)) in locations {
			let id = &image_id.id;
			let image = data.get(id).unwrap();
			let x = location.x();
			let y = location.y();
			let width = location.width();
			let height = location.height();

			for i in 0..mipmaps {
				let pixels = image.get_pixels(i);
				//gl::pixel_store_i(gl::UNPACK_ROW_LENGTH, width);
				//gl::pixel_store_i(gl::UNPACK_IMAGE_HEIGHT, height / 2);
				gl::pixel_store_i(gl::UNPACK_ALIGNMENT, 1);
				gl::tex_parameter_i(
					gl::TEXTURE_2D,
					gl::TEXTURE_MIN_FILTER,
					gl::NEAREST_MIPMAP_LINEAR,
				);
				gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST);
				gl::tex_sub_image_2d(
					gl::TEXTURE_2D,
					i as i32,
					x >> i,
					y >> i,
					width >> i,
					height >> i,
					gl::RGBA,
					gl::UNSIGNED_BYTE,
					pixels.as_ptr(),
				);
			}

			// TODO make images 1 off so i dont need to have this offset
			images.insert(
				*(id),
				AtlasImage {
					x: Self::gl_pos(atlas_width, x as f32),
					y: Self::gl_pos(atlas_height, y as f32),
					width: Self::gl_pos(atlas_width, width as f32),
					height: Self::gl_pos(atlas_height, height as f32),
				},
			);
		}

		//for i in 0..mipmaps {
		//	println!("savign thigs {}", i);
		//	let pbo = gl::gen_buffer();
		//	gl::bind_buffer(&BufferType::PixelPackBuffer, &pbo);
		//	let width = (atlas_width >> i);
		//	let height = (atlas_height >> i);
		//	let size = (width * height * 4) as usize;
		//	gl::buffer_data::<u8>(BufferTarget::PixelPackBuffer, size, None, BufferUsage::StreamRead);
		//	gl::get_tex_image(gl::TEXTURE_2D, i as i32, gl::RGBA, width, height, 0, gl::UNSIGNED_BYTE);
		//
		//
		//	let mut out: Vec<u8> = Vec::with_capacity(size);
		//	for j in 0..size {
		//		out.push(0);
		//	}
		//	gl::get_buffer_subdata(gl::PIXEL_PACK_BUFFER, 0, size, &mut out);
		//
		//	image::save_buffer(format!("C:\\Program Files (x86)\\inkscape\\cppProjects\\rustaria\\archive\\{}-tile-atlas.png", i), out.as_slice(), width, height, ColorType::Rgba8);
		//	gl::delete_buffer(pbo);
		//}

		Self { id, images }
	}

	pub fn bind(&self) {
		gl::active_texture(gl::TEXTURE0);
		gl::bind_texture(gl::TEXTURE_2D, self.id);
	}

	fn gl_pos(size: u32, pos: f32) -> f32 {
		pos as f32 / size as f32
	}
}

pub struct AtlasImage {
	pub x: f32,
	pub y: f32,
	pub width: f32,
	pub height: f32,
}
