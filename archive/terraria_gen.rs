use std::collections::HashMap;
use std::detect::__is_feature_detected::avx;

use image::math::utils::clamp;
use rand::Rng;

use crate::consts::TileId;
use crate::world::{Chunk, CHUNK_SIZE, World};
use crate::world::pos::{ChunkPos, ChunkTilePos, TilePos};
use crate::world::tile::Tile;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum FeatureType {
    Plateau = 0,
    Hill = 1,
    Dale = 2,
    Mountain = 3,
    Valley = 4,
}

impl FeatureType {
    pub fn new(id: i32) -> FeatureType {
        match id {
            0 => FeatureType::Plateau,
            1 => FeatureType::Hill,
            2 => FeatureType::Dale,
            3 => FeatureType::Mountain,
            4 => FeatureType::Valley,
            _ => panic!("what")
        }
    }
}

const DRUNK_WORLD_GEN: bool = false;
const GOOD_WORLD_GEN: bool = false;

pub struct SurfaceHistory {
    heights: Vec<f32>,
    current_index: i32,
}

impl SurfaceHistory {
    pub fn new(size: i32) -> SurfaceHistory {
        let mut vec = Vec::with_capacity(size as usize);
        for i in 0..size {
            vec.push(0f32);
        }
        Self {
            heights: vec,
            current_index: 0,
        }
    }

    pub fn record(&mut self, value: f32) {
        self.heights[self.current_index as usize] = value;
        self.current_index = (self.current_index + 1) as i32 % self.heights.capacity() as i32;
    }

    pub fn get(&self, index: i32) -> f32 {
        self.heights[((index + self.current_index) as i32 % self.heights.len() as i32) as usize]
    }

    pub fn set(&mut self, index: i32, value: f32) {
        let current_index = self.current_index;
        let cap = self.heights.len();
        self.heights[((index + current_index) as i32 % cap as i32) as usize] = value;
    }

    pub fn length(&self) -> usize {
        self.heights.capacity()
    }
}


pub fn random(v1: i32, v2: i32) -> i32 {
    rand::thread_rng().gen_range(v1..v2) as i32
}

pub fn random_f(v1: i32, v2: i32) -> f32 {
    rand::thread_rng().gen_range(v1..v2) as f32
}


pub struct WorldGen {
    max_tiles_y: i32,
    max_tiles_x: i32,
    world: Vec<Vec<Tile>>,
    flat_beach_padding: i32,
    world_surface: f32,
    world_surface_high: f32,
    world_surface_low: f32,
    rock_layer: f32,
    rock_layer_high: f32,
    rock_layer_low: f32,
    water_line: i32,
    lava_line: i32,
    left_beach_size: i32,
    right_beach_size: i32,
}

impl WorldGen {
    pub fn new(max_tiles_y: i32, max_tiles_x: i32) -> WorldGen {
        let mut world: Vec<Vec<Tile>> = Vec::with_capacity(max_tiles_y as usize);
        for y in 0..max_tiles_y {
            let mut row = Vec::with_capacity(max_tiles_x as usize);
            for x in 0..max_tiles_x {
                row.push(Tile::id(TileId { id: 0 }))
            }
            world.push(row);
        }


        Self {
            max_tiles_y,
            max_tiles_x,
            world,
            flat_beach_padding: 0,
            world_surface: 0.0,
            world_surface_high: 0.0,
            world_surface_low: 0.0,
            rock_layer: 0.0,
            rock_layer_high: 0.0,
            rock_layer_low: 0.0,
            water_line: 0,
            lava_line: 0,
            left_beach_size: 0,
            right_beach_size: 0,
        }
    }

    pub fn export(&mut self) -> World {
        let mut chunks: HashMap<ChunkPos, Chunk> = HashMap::new();
        let chunks_x = (self.max_tiles_x as f32 / CHUNK_SIZE as f32).ceil();
        let chunks_y = (self.max_tiles_y as f32 / CHUNK_SIZE as f32).ceil();
        for chunk_y in 0..(chunks_y as u16) {
            for chunk_x in 0..(chunks_x as i16) {
                let pos = ChunkPos::new(chunk_x, chunk_y);
                let mut chunk = Chunk::new(pos);
                let mut tiles = chunk.solid_tiles;
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let tile_x = (chunk_x as i32 * CHUNK_SIZE as i32) + x as i32;
                        let tile_y = (chunk_y as i32 * CHUNK_SIZE as i32) + y as i32;
                        if tile_x < self.max_tiles_x && tile_y < self.max_tiles_y {
                            let tile = self.get(tile_x, (self.max_tiles_y - 1) - tile_y);
                            tiles[y][x] = tile;
                        }
                    }
                }

                chunk.solid_tiles = tiles;

                chunks.insert(pos, chunk);
            }
        };

        World::import(chunks)
    }

    pub fn generate_terrain(&mut self) {
        let max_tiles_y = self.max_tiles_y as f32;
        let max_tiles_x = self.max_tiles_x as f32;
        let left_beach_size = 100;
        let right_beach_size = 100;
        let flat_beach_padding = 5;

        let beach_padding = flat_beach_padding;
        // progress.Message = Lang.gen[0].Value;
        let mut feature_type = FeatureType::Plateau;


        let mut surface_layer = (max_tiles_y as f32 * 0.3f32 * (random_f(90, 110) as f32 * 0.005f32));
        let mut rock_layer = (surface_layer as f32 + max_tiles_y as f32 * 0.2f32) * (random_f(90, 110) as f32 * 0.01f32);
        let mut lowest_surface_level: f32 = surface_layer;
        let mut highest_surface_level: f32 = surface_layer;
        let mut lowest_rock_level: f32 = rock_layer;
        let mut highest_rock_level: f32 = rock_layer;
        let target_height = max_tiles_y as f32 * 0.23f32;

        let mut history = SurfaceHistory::new(500);
        let mut terrain_start: f32 = (left_beach_size + beach_padding) as f32;


        for x in 0..(max_tiles_x as i32) {
            lowest_surface_level = surface_layer.min(lowest_surface_level);
            highest_surface_level = surface_layer.max(highest_surface_level);
            lowest_rock_level = rock_layer.min(lowest_rock_level);
            highest_rock_level = rock_layer.max(highest_rock_level);
            if terrain_start <= 0 as f32 {
                feature_type = FeatureType::new(random(0, 5));
                terrain_start = random(5, 40) as f32;
                if feature_type == FeatureType::Plateau {
                    terrain_start *= random_f(5, 30) as f32 * 0.2f32;
                }
            }
            terrain_start -= 1f32;
            if (x > (max_tiles_x as f32 * 0.45) as i32 && x < (max_tiles_x as f32 * 0.55) as i32) && (feature_type == FeatureType::Mountain || feature_type == FeatureType::Valley) {
                feature_type = FeatureType::new(random(0, 3));
            }

            if x > (max_tiles_x as f32 * 0.48) as i32 && x < (max_tiles_x as f32 * 0.52) as i32 {
                feature_type = FeatureType::Plateau;
            }

            surface_layer += Self::generate_world_surface_offset(&feature_type) as f32;

            let mut num6 = 0.17f32;
            let mut num7 = 0.26f32;
            if (DRUNK_WORLD_GEN) {
                num6 = 0.15f32;
                num7 = 0.28f32;
            }

            if x < (left_beach_size + beach_padding) as i32 || x > (max_tiles_x as i32 - right_beach_size - beach_padding) as i32 {
                surface_layer = surface_layer.clamp(max_tiles_y * 0.17, target_height);
            } else if surface_layer < max_tiles_y * num6 {
                surface_layer = max_tiles_y * num6;
                terrain_start = 0f32;
            } else if surface_layer > max_tiles_y * num7 {
                surface_layer = max_tiles_y * num7;
                terrain_start = 0f32;
            }
            while random(0, 3) == 0 {
                rock_layer += random_f(-2, 3);
            }
            if rock_layer < surface_layer + max_tiles_y * 0.06 {
                rock_layer += 1f32;
            }

            if rock_layer > surface_layer + max_tiles_y * 0.35 {
                rock_layer -= 1f32;
            }
            history.record(surface_layer);

            self.fill_column(x, surface_layer, rock_layer);
            if x == (max_tiles_x as i32 - right_beach_size - beach_padding) as i32 {
                if lowest_surface_level > target_height {
                    self.retarget_surface_history(&mut history, x, target_height);
                }
                feature_type = FeatureType::Plateau;
                terrain_start = max_tiles_x - x as f32;
            }
        }

        self.world_surface = highest_surface_level + 25.0;
        self.rock_layer = highest_rock_level;
        let num8 = (((self.rock_layer - self.world_surface) / 6.0f32) * 6f32);
        self.rock_layer = (self.world_surface + num8).round();
        let water_line: i32 = ((self.rock_layer + max_tiles_y) / 2f32) as i32 + random(-100, 20);
        let lava_line: i32 = water_line + random(50, 80);
        let surface_offset: i32 = 20;
        if lowest_rock_level < highest_surface_level + surface_offset as f32 {
            let num12 = (lowest_rock_level + highest_surface_level) / 2.0;
            let mut num13 = (lowest_rock_level - highest_surface_level).abs();
            if num13 < surface_offset as f32 {
                num13 = surface_offset as f32;
            }
            lowest_rock_level = num12 + num13 / 2.0;
            highest_surface_level = num12 - num13 / 2.0;
        }
        self.rock_layer = rock_layer;
        self.rock_layer_high = highest_rock_level;
        self.rock_layer_low = lowest_rock_level;
        self.world_surface = surface_layer;
        self.world_surface_high = highest_surface_level;
        self.world_surface_low = lowest_surface_level;
        self.water_line = water_line;
        self.lava_line = lava_line;
    }

    fn set(&mut self, x: i32, y: i32, id: u32) {
        self.world[y as usize][x as usize] = Tile::id(TileId { id })
    }

    fn get(&mut self, x: i32, y: i32) -> Tile {
        self.world[y as usize][x as usize]
    }

    fn retarget_column(&mut self, x: i32, world_surface: f32) {
        for y in 0..(world_surface as i32) {
            self.set(x, y, 0);
        }

        for y in (world_surface as i32)..(self.max_tiles_y) {
            let i = self.get(x, y).id.id;
            if i != 2 || i != 0 {
                self.set(x, y, 1);
            }
        }
    }

    pub fn retarget_surface_history(&mut self, history: &mut SurfaceHistory, target_x: i32, target_height: f32) {
        let mut index1 = 0usize;
        while index1 < history.length() / 2 && history.get((history.length() - 1) as i32) > target_height {
            for index2 in 0..(history.length() - index1 * 2) {
                let num = history.get((history.length() - index2 - 1) as i32) - 1.0;
                history.set((history.length() - index2 - 1) as i32, num);
                if num <= target_height {
                    break;
                }
            }
            index1 += 1;
        }
        for i in 0..history.length() {
            let world_surface = history.get((history.length() - i - 1) as i32);
            self.retarget_column(target_x - i as i32, world_surface);
        }
    }

    pub fn fill_column(&mut self, x: i32, world_surface: f32, rock_layer: f32) {
        for y in 0..(world_surface as i32) {
            self.set(x, y, 0);
        }

        for y in (world_surface as i32)..self.max_tiles_y {
            if (y as f32) < (rock_layer) {
                self.set(x, y, 1);
            } else {
                self.set(x, y, 2);
            }
        }
    }

    pub fn generate_world_surface_offset(feature_type: &FeatureType) -> f32 {
        let mut world_surface_offset = 0.0;
        if (DRUNK_WORLD_GEN || GOOD_WORLD_GEN) && random(0, 2) == 0 {
            match feature_type {
                FeatureType::Plateau => {
                    while random(0, 6) == 0 {
                        world_surface_offset += random_f(-1, 2);
                    }
                }
                FeatureType::Hill => {
                    while random(0, 3) == 0 {
                        world_surface_offset -= 1f32;
                    }
                    while random(0, 10) == 0 {
                        world_surface_offset += 1f32;
                    }
                }
                FeatureType::Dale => {
                    while random(0, 3) == 0 {
                        world_surface_offset += 1f32;
                    }
                    while random(0, 10) == 0 {
                        world_surface_offset -= 1f32;
                    }
                }
                FeatureType::Mountain => {
                    while random(0, 3) != 0 {
                        world_surface_offset -= 1f32;
                    }
                    while random(0, 6) == 0 {
                        world_surface_offset += 1f32;
                    }
                }
                FeatureType::Valley => {
                    while random(0, 3) != 0 {
                        world_surface_offset += 1f32;
                    }
                    while random(0, 5) == 0 {
                        world_surface_offset -= 1f32;
                    }
                }
            }
        } else {
            match feature_type {
                FeatureType::Plateau => {
                    while random(0, 7) == 0 {
                        world_surface_offset += random_f(-1, 2);
                    }
                }
                FeatureType::Hill => {
                    while random(0, 4) == 0 {
                        world_surface_offset -= 1f32;
                    }
                    while random(0, 10) == 0 {
                        world_surface_offset += 1f32;
                    }
                }
                FeatureType::Dale => {
                    while random(0, 4) == 0 {
                        world_surface_offset += 1f32;
                    }
                    while random(0, 10) == 0 {
                        world_surface_offset -= 1f32;
                    }
                }
                FeatureType::Mountain => {
                    while random(0, 2) == 0 {
                        world_surface_offset -= 1f32;
                    }
                    while random(0, 6) == 0 {
                        world_surface_offset += 1f32;
                    }
                }
                FeatureType::Valley => {
                    while random(0, 2) == 0 {
                        world_surface_offset += 1f32;
                    }
                    while random(0, 5) == 0 {
                        world_surface_offset -= 1f32;
                    }
                }
            }
        }
        world_surface_offset
    }
}

