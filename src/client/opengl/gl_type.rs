use std::any::TypeId;
use glam::{Vec2, Vec3, Vec4};

use opengl_raw::gll::types::GLenum;

use crate::client::opengl::gl::DataType;

use super::gl;


// ======================================== TYPE ========================================
pub trait GlType {
    fn get_size() -> usize;
    fn match_gl(gl_enum: GLenum) -> bool;
    fn define_attrib_structure(index: u32);
}

macro_rules! prim {
    ($([$DATA_TYPE:expr, $AMOUNT:literal] ($TYPE:ty | $GL_ENUM:expr)),*) => {
        $(impl GlType for $TYPE {
            fn get_size() -> usize {
                std::mem::size_of::<$TYPE>()
            }

            fn match_gl(gl_enum: GLenum) -> bool {
                gl_enum == $GL_ENUM
            }

            fn define_attrib_structure(index: u32) {
                gl::vertex_attrib_pointer(index, $AMOUNT, &$DATA_TYPE, false,  $DATA_TYPE.get_size() * $AMOUNT, 0);
            }
        }
        )*
    };
}
prim!(
    [DataType::UInt, 1] (u32 | gl::UNSIGNED_INT),
    [DataType::UInt, 2] ((u32, u32) |gl::UNSIGNED_INT_VEC2),
    [DataType::UInt, 3] ((u32, u32, u32) | gl::UNSIGNED_INT_VEC3),
    [DataType::UInt, 4] ((u32, u32, u32, u32) | gl::UNSIGNED_INT_VEC4),
    [DataType::Bool, 1] (bool | gl::BOOL),
    [DataType::Bool, 2] ((bool, bool) | gl::BOOL_VEC2),
    [DataType::Bool, 3] ((bool, bool, bool) | gl::BOOL_VEC3),
    [DataType::Bool, 4] ((bool, bool, bool, bool) | gl::BOOL_VEC4),
    [DataType::Double, 1] (f64 | gl::DOUBLE),
    [DataType::Double, 2] ((f64, f64) | gl::DOUBLE_VEC2),
    [DataType::Double, 3] ((f64, f64, f64) | gl::DOUBLE_VEC3),
    [DataType::Double, 4] ((f64, f64, f64, f64) | gl::DOUBLE_VEC4),
    [DataType::Float, 1] (f32 | gl::FLOAT),
    [DataType::Float, 2] ((f32, f32) | gl::FLOAT_VEC2),
    [DataType::Float, 3] ((f32, f32, f32) | gl::FLOAT_VEC3),
    [DataType::Float, 4] ((f32, f32, f32, f32) | gl::FLOAT_VEC4),
    [DataType::Float, 2] (Vec2 | gl::FLOAT_VEC2),
    [DataType::Float, 3] (Vec3 | gl::FLOAT_VEC3),
    [DataType::Float, 4] (Vec4 | gl::FLOAT_VEC4),
    [DataType::IInt, 1] (i32 | gl::INT),
    [DataType::IInt, 2] ((i32, i32) | gl::INT_VEC2),
    [DataType::IInt, 3] ((i32, i32, i32) | gl::INT_VEC3),
    [DataType::IInt, 4] ((i32, i32, i32, i32) | gl::INT_VEC4)
);