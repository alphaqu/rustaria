use std::ffi::CStr;
use std::ptr::null;
use std::sync::mpsc::Receiver;

use glfw::{Glfw, Window};

use crate::{attach_shader, compile_shader, create_shader, delete_program, detach_shader, FRAGMENT_SHADER, gl, GLchar, GLenum, gll, GLsizei, GLuint, link_program, shader_source, Tile, VERTEX_SHADER};

pub struct Renderer {
    glfw: Glfw,
    window: Window,
    event: Receiver<()>,
}

pub struct Program {
    program_id: GLuint,
    fragment_id: GLuint,
    vertex_id: GLuint,
}

impl Program {
    fn new(fragment: String, vertex: String) -> Program {
        let program_id = gl::create_program();
        let fragment_id = Self::create_shader(program_id, FRAGMENT_SHADER, fragment);
        let vertex_id = Self::create_shader(program_id, VERTEX_SHADER, vertex);

        Self {
            program_id,
            fragment_id,
            vertex_id,
        }
    }

    fn create_shader(program_id: GLuint, shader: GLenum, code: String) -> GLuint {
        let shader_id = create_shader(shader);
        shader_source(shader_id, code.as_str());
        compile_shader(shader_id);
        attach_shader(program_id, shader_id);
        shader_id
    }

    fn attach(&self) {
        attach_shader(self.program_id, self.vertex_id);
        attach_shader(self.program_id, self.fragment_id);
        link_program(self.program_id);
    }

    fn detach(&self) {
        detach_shader(self.program_id, self.vertex_id);
        detach_shader(self.program_id, self.fragment_id);
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        if self.program_id != 0 {
            delete_program(self.program_id);
        }
    }
}


impl Renderer {

}

pub struct TileRenderer {
    program: Program
}

impl TileRenderer {
    
}