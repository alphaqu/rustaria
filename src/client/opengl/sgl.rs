use std::collections::HashMap;
use std::marker::PhantomData;

use opengl_raw::gll::types::{GLenum, GLint, GLuint};

use crate::client::opengl::gl::{BufferType, BufferUsage, DataType, VertexDivisor};
use crate::client::opengl::gl_type::GlType;

use super::gl;

pub type GlId = GLuint;


// ======================================== PROGRAM ========================================
pub struct Program {
    id: GlId,
    // Shaders
    vertex_shader: Shader,
    fragment_shader: Shader,
    geometry_shader: Option<Shader>,

    // Other
    attributes: HashMap<String, RawAttribute>,
    uniforms: HashMap<String, RawUniform>,
}

impl Program {
    pub fn create(vertex_code: String, fragment_code: String, geometry_code: Option<String>) -> Self {
        // Create OpenGL Program
        let id = gl::create_program();

        // Create Shaders. Geometry is optional
        let vertex_id = Self::create_shader(id, gl::VERTEX_SHADER, vertex_code);
        let fragment_id = Self::create_shader(id, gl::FRAGMENT_SHADER, fragment_code);
        let geometry_id = geometry_code.map(|code| Self::create_shader(id, gl::GEOMETRY_SHADER, code));

        // Link stuff
        gl::link_program(id);
        if gl::get_program_iv(id, gl::LINK_STATUS) == 0 {
            panic!("Could not link program. Error: {}", gl::get_program_info_log(id));
        }

        // Detach Shaders
        if vertex_id != 0 {
            gl::detach_shader(id, vertex_id);
        }

        if fragment_id != 0 {
            gl::detach_shader(id, fragment_id);
        }

        geometry_id.map(|geometry_id| {
            if geometry_id != 0 {
                gl::detach_shader(id, geometry_id);
            }
        });

        // Validate Program so nothing blows up.
        gl::validate_program(id);
        if gl::get_program_iv(id, gl::VALIDATE_STATUS) == 0 {
            panic!("Could not validate program. Error: {}", gl::get_program_info_log(id));
        }

        // Fill attributes
        let mut attributes = HashMap::new();
        let count = gl::get_program_iv(id, gl::ACTIVE_ATTRIBUTES);
        for i in 0..count {
            let (_, var_type, name) = gl::get_active_attrib(id, i as u32, 16);
            println!("Attribute {}", name);
            attributes.insert(name.clone(), RawAttribute {
                location: i as u32,
                var_type,
                name,
            });
        }

        // Fill uniforms
        let mut uniforms = HashMap::new();
        let count = gl::get_program_iv(id, gl::ACTIVE_UNIFORMS);
        for i in 0..count {
            let (_, var_type, name) = gl::get_active_uniform(id, i as u32, 16);
            println!("Uniform {}", name);
            uniforms.insert(name.clone(), RawUniform {
                location: i,
                var_type,
                name,
            });
        }

        Self {
            id,
            vertex_shader: Shader { id: vertex_id },
            fragment_shader: Shader { id: fragment_id },
            geometry_shader: geometry_id.map(|id| Shader { id }),
            attributes,
            uniforms,
        }
    }

    pub fn create_shader(program_id: GLuint, shader: GLenum, code: String) -> GLuint {
        let id = gl::create_shader(shader);
        if id == 0 {
            panic!("Could not create shader. Type: {}", shader);
        }

        gl::shader_source(id, code);

        gl::compile_shader(id, program_id);
        if gl::get_shader_iv(id, gl::COMPILE_STATUS) == 0 {
            panic!("Could not compile shader. {}", gl::get_shader_info_log(id))
        }

        gl::attach_shader(program_id, id);
        id
    }


    pub fn get_attribute<V: GlType>(&self, name: &str) -> Attribute<V> {
        let option = self.attributes.get(name).expect(&*format!("Could not find attribute {}", name));


        if !V::match_gl(option.var_type) {
            panic!("Type does not match");
        };

        Attribute {
            index: option.location,
            data_type: PhantomData::default(),
            name: option.name.clone(),
        }
    }

    pub fn get_uniform<V: GlType>(&self, name: &str) -> Uniform<V> {
        let option = self.uniforms.get(name).expect("Could not find uniform");

        if !V::match_gl(option.var_type) {
            panic!("Type does not match {} in uniform {}", option.var_type, name);
        };

        Uniform {
            location: option.location,
            data_type: PhantomData::default(),
            name: option.name.clone(),
        }
    }

    pub fn bind(&self) {
        gl::use_program(Some(self.id));
    }

    pub fn unbind(&self) {
        gl::use_program(None);
    }
}

// ======================================== VAO / VBO ========================================
/// VertexData > (in gl language) > Vertex Array Objects
pub struct VertexData {
    id: GlId,
    arrays: Vec<RawVertexArray>,
}

impl VertexData {
    pub fn new(max_id: u32) -> Self {
        let mut arrays = Vec::with_capacity(max_id as usize);

        for _ in 0..max_id {
            arrays.push(RawVertexArray { index: 0, id: 0 })
        }

        Self {
            id: gl::gen_vertex_arrays(),
            arrays
        }
    }

    pub fn add_vertex_array<V: GlType>(&mut self, attribute: &Attribute<V>, data: Vec<V>, usage: BufferUsage, divisor: VertexDivisor) {
        // Bind VAO
        gl::bind_vertex_array(Some(self.id));

        // Bind VBO
        let id = gl::gen_buffer();
        gl::bind_buffer(BufferType::ArrayBuffer, Some(id));

        // Upload data
        gl::buffer_data_vec(BufferType::ArrayBuffer, &data, usage);

        // Define structure and define its divisor
        V::define_attrib_structure(attribute.index);
        gl::vertex_attrib_divisor(attribute.index, divisor);


        // Unbind VBO / VAO
        gl::bind_buffer(BufferType::ArrayBuffer, None);
        gl::bind_vertex_array(None);

        self.arrays.insert(attribute.index as usize, RawVertexArray { index: attribute.index, id })
    }

    pub fn bind(&self) {
        gl::bind_vertex_array(Some(self.id));
        for x in &self.arrays {
            gl::enable_vertex_attribute_array(x.index);
        }
    }

    pub fn unbind(&self) {
        for x in &self.arrays {
            gl::disable_vertex_attribute_array(x.index);
        }
        gl::bind_vertex_array(None);
    }
}

impl Drop for VertexData {
    fn drop(&mut self) {
        gl::bind_vertex_array(Some(0));
        gl::delete_vertex_array(self.id);
    }
}


/// VertexArray > (in gl language) > Vertex Buffer Object
struct RawVertexArray {
    index: u32,
    id: GlId,
}

impl Drop for RawVertexArray {
    fn drop(&mut self) {
        gl::disable_vertex_attribute_array(self.index);
        gl::bind_buffer(BufferType::ArrayBuffer, Some(0));
        gl::delete_buffer(self.id);
    }
}

// ======================================== ATTRIBUTE ========================================
// layout (location=$location) in $data_type $name;
pub struct Attribute<V: GlType> {
    index: u32,
    data_type: PhantomData<V>,
    name: String,
}

struct RawAttribute {
    location: u32,
    var_type: GLenum,
    name: String,
}

// ======================================== SHADER ========================================
pub struct Shader {
    id: GlId,
}


// ======================================== UNIFORM ========================================
pub struct Uniform<V> {
    pub location: i32,
    data_type: PhantomData<V>,
    name: String,
}

struct RawUniform {
    location: i32,
    var_type: GLenum,
    name: String,
}


pub trait UniformType<V> {
    fn apply(&self, value: V);
}

macro_rules! simple_uniform {
    ($TYPE:ty, $METHOD:ident) => {
        impl UniformType<$TYPE> for Uniform<$TYPE> {
            fn apply(&self, value: $TYPE) {
                super::gl::$METHOD(self.location, value);
            }
        }
    };
}

macro_rules! multi_uniform {
    ($TYPE:ty, ($($NAME:ident),*), $METHOD:ident) => {
        impl UniformType<$TYPE> for Uniform<$TYPE> {
            fn apply(&self, ($($NAME),*): $TYPE) {
                super::gl::$METHOD(self.location, $($NAME),*);
            }
        }
    };
}

simple_uniform!(u32, uniform_1ui);
simple_uniform!(i32, uniform_1i);
simple_uniform!(f32, uniform_1f);
simple_uniform!(f64, uniform_1d);
multi_uniform!((u32, u32), (v0, v1), uniform_2ui);
multi_uniform!((i32, i32), (v0, v1), uniform_2i);
multi_uniform!((f32, f32), (v0, v1), uniform_2f);
multi_uniform!((f64, f64), (v0, v1), uniform_2d);
multi_uniform!((u32, u32, u32), (v0, v1, v2), uniform_3ui);
multi_uniform!((i32, i32, i32), (v0, v1, v2), uniform_3i);
multi_uniform!((f32, f32, f32), (v0, v1, v2), uniform_3f);
multi_uniform!((f64, f64, f64), (v0, v1, v2), uniform_3d);
multi_uniform!((u32, u32, u32, u32), (v0, v1, v2, v3), uniform_4ui);
multi_uniform!((i32, i32, i32, i32), (v0, v1, v2, v3), uniform_4i);
multi_uniform!((f32, f32, f32, f32), (v0, v1, v2, v3), uniform_4f);
multi_uniform!((f64, f64, f64, f64), (v0, v1, v2, v3), uniform_4d);