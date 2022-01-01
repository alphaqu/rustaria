#![allow(dead_code)]
#![allow(unused_variables)]

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::iter::Map;
use std::path::Path;

use image::{ColorType, DynamicImage, GenericImage, GenericImageView, ImageBuffer};
use image::imageops::FilterType;
use opengl_raw::gll::types::{GLenum, GLuint};
use rectangle_pack::{contains_smallest_box, GroupedRectsToPlace, pack_rects, RectanglePackError, RectanglePackOk, RectToInsert, TargetBin, volume_heuristic};

use crate::client::opengl::builder::VertexBuilderTrait;
// use crate::client::opengl::gl;
use crate::client::opengl::gl::{
    self, BufferType, BufferUsage, DataType, VertexDivisor,
};
use crate::client::opengl::gl_type::GlType;
use crate::client::opengl::sgl::{Uniform, UniformType};
use crate::world::tile::TileId;
use crate::world::wall::WallId;

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

        //for i in 0..=mipmaps {
        //    println!("savign thigs {}", i);
        //    let pbo = gl::gen_buffer();
        //    gl::bind_buffer(BufferType::PixelPackBuffer, Some(pbo));
        //    let width = (atlas_w >> i);
        //    let height = (atlas_h >> i);
        //    let size = (width * height * 4) as usize;
        //    gl::buffer_data::<u8>(BufferType::PixelPackBuffer, size, None, BufferUsage::StreamRead);
        //    gl::get_tex_image(gl::TEXTURE_2D, i as i32, gl::RGBA, width, height, 0, gl::UNSIGNED_BYTE);
        //    let mut out: Vec<u8> = Vec::with_capacity(size);
        //    for j in 0..size {
        //        out.push(0);
        //    }
        //    gl::get_buffer_subdata(gl::PIXEL_PACK_BUFFER, 0, size, &mut out);
        //    image::save_buffer(format!("./{}-tile-atlas.png", i), out.as_slice(), width, height, ColorType::Rgba8);
        //    gl::delete_buffer(pbo);
        //}

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
        gl::bind_texture(gl::TEXTURE_2D, self.id);
        gl::active_texture(gl::TEXTURE0);
    }

    fn gl_pos(size: u32, pos: f32) -> f32 {
        pos as f32 / size as f32
    }
}

pub struct Sampler2d {
    id: i32,
}

impl Sampler2d {
    pub fn new(id: i32) -> Self {
        Self {
            id
        }
    }
}

impl GlType for Sampler2d {
    fn get_size() -> usize {
        4
    }

    fn match_gl(gl_enum: GLenum) -> bool {
        gl_enum == gl::SAMPLER_2D
    }

    fn define_attrib_structure(index: u32) {
        gl::vertex_attrib_pointer(index, 4, &DataType::IInt, false, 0, 0);
    }
}

impl UniformType<Sampler2d> for Uniform<Sampler2d> {
    fn apply(&self, value: Sampler2d) {
        gl::uniform_1i(self.location, value.id);
    }
}

pub struct AtlasImage {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
