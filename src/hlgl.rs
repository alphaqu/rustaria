#![allow(dead_code)]
#![allow(unused_variables)]

use std::ops::Index;

use crate::{BufferTarget, BufferType, BufferUsage, DataType, gl, GLenum, GLint, GLuint, VertexDivisor};

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
    pub fn new(size: usize) -> VertexLayout {
        let id = gl::gen_vertex_arrays();

        Self { id, vertex_buffers: Vec::with_capacity(size) }
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

pub struct VertexBuilder<'a> {
    data: Vec<f32>,
    viewport: &'a Viewport,
}

impl<'a> VertexBuilder<'a> {
    pub fn new(viewport: &Viewport) -> VertexBuilder {
        VertexBuilder { data: Vec::new(), viewport }
    }

    pub fn quad(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.pos_2d(x, y + height);
        self.pos_2d(x, y);
        self.pos_2d(x + width, y + height);
        self.pos_2d(x + width, y + height);
        self.pos_2d(x, y);
        self.pos_2d(x + width, y);
    }

    pub fn pos_2d(&mut self, x: f32, y: f32) {
        self.pos_x(x);
        self.pos_y(y);
    }

    pub fn pos_x(&mut self, x: f32) {
        self.add(self.viewport.gl_get_x(x));
    }

    pub fn pos_y(&mut self, y: f32) {
        self.add(self.viewport.gl_get_y(y));
    }

    pub fn add(&mut self, value: f32) {
        self.data.push(value);
    }

    pub fn size(&self) -> u32 {
        self.data.len() as u32
    }

    pub fn export(self) -> Vec<f32> {
        self.data
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

pub struct Viewport {
    width: i32,
    height: i32,
}

impl Viewport {
    pub fn new(width: i32, height: i32) -> Viewport {
        Viewport { width, height }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        gl::viewport(0, 0, width, height);
        self.width = width;
        self.height = height;
    }

    pub fn gl_get_x(&self, x: f32) -> f32 {
        (-(x / self.width as f32) - 0.5f32) * 2f32
    }

    pub fn gl_get_y(&self, y: f32) -> f32 {
        (-(y / self.height as f32) - 0.5f32) * 2f32
    }

    pub fn get_width(&self) -> u32 {
        self.width as u32
    }

    pub fn get_height(&self) -> u32 {
        self.height as u32
    }
}
