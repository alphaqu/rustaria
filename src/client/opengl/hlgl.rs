#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::{BTreeMap, HashMap};
use std::ops::Index;

use image::{DynamicImage, GenericImageView};
use rectangle_pack::{contains_smallest_box, GroupedRectsToPlace, pack_rects, RectToInsert, TargetBin, volume_heuristic};

use crate::client::opengl::gl;
use crate::client::opengl::gl::{BufferTarget, BufferType, BufferUsage, DataType, VertexDivisor};
use crate::gll::types::{GLenum, GLuint};
use crate::registry::TileId;

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

        Self { id, vertex_buffers: Vec::with_capacity(buffers) }
    }

    pub fn add_vbo<T>(&mut self, index: u32, data: Vec<T>, amount: i32, usage: BufferUsage, data_type: DataType, instance: VertexDivisor) {
        gl::bind_vertex_array(&self.id);
        let vbo = VertexBuffer::new(data, amount, usage, data_type);
        let data1 = &vbo.data_type;
        gl::vertex_attrib_pointer(
            index,
            vbo.amount,
            data1,
            false,
            data1.get_size() * amount,
        );
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
    pub fn new<T>(data: Vec<T>, amount: i32, usage: BufferUsage, data_type: DataType) -> VertexBuffer {
        let id = gl::gen_buffer();
        gl::bind_buffer(&BufferType::ArrayBuffer, &id);
        gl::buffer_data(BufferTarget::ArrayBuffer, &data, usage);
        Self { id, amount, element_size: std::mem::size_of::<T>(), data_type, buffer_type: BufferType::ArrayBuffer }
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
    Tiles
}

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub struct Atlas {
    id: GLuint,
    images: HashMap<u32, AtlasImage>,
}

impl Atlas {
    pub fn get_image(&self, id: TileId) -> &AtlasImage {
        self.images.get(&id.id).expect("Could not find image")
    }

    pub fn create(data: HashMap<u32, Image>, atlas_width: u32, atlas_height: u32, group: AtlasGroup) -> Atlas {
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
        ).unwrap();


        println!("Allocating {} images", &data.len());
        let id = gl::gen_texture();

        let mipmaps = 3;

        gl::bind_texture(gl::TEXTURE_2D, id);
        gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAX_LOD, mipmaps);
        gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_LOD, 0);
        gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, mipmaps);
        gl::tex_parameter_f(gl::TEXTURE_2D, gl::TEXTURE_LOD_BIAS, 0.2f32);

        for i in 0..(mipmaps + 1) {
            gl::tex_image_2d(gl::TEXTURE_2D, i as i32, gl::RGBA, atlas_width >> i, atlas_height >> i, 0, gl::RGBA, gl::UNSIGNED_BYTE, Option::None);
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

            for i in 0..(mipmaps + 1) {
                Self::upload_sub_image(&image, i, x >> i, y >> i, 0, 0, width >> i, height >> i);
            }

            // TODO make images 1 off so i dont need to have this offset
            images.insert(*(id), AtlasImage {
                x: Self::gl_pos(atlas_width, x as f32),
                y: Self::gl_pos(atlas_height, y as f32),
                width: Self::gl_pos(atlas_width, width as f32),
                height: Self::gl_pos(atlas_height, height as f32 - 0.1f32),
            });
        };
        Self {
            id,
            images,
        }
    }

    fn upload_sub_image(image: &Image, level: u32, offset_x: u32, offset_y: u32, unpack_skip_pixels: u32, unpack_skip_rows: u32, width: u32, height: u32) {
        gl::pixel_store_i(gl::UNPACK_ROW_LENGTH, width << level);
        gl::pixel_store_i(gl::UNPACK_SKIP_PIXELS, unpack_skip_pixels);
        gl::pixel_store_i(gl::UNPACK_SKIP_ROWS, unpack_skip_rows);
        gl::pixel_store_i(gl::UNPACK_ALIGNMENT, 1);
        gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S,     gl::CLAMP_TO_EDGE);
        gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T,     gl::CLAMP_TO_EDGE);
        gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_LINEAR);
        gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST);
        gl::tex_sub_image_2d(gl::TEXTURE_2D, level as i32, offset_x, offset_y, width, height, gl::RGBA, gl::UNSIGNED_BYTE, &image.data);
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