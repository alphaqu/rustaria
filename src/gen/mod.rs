use std::collections::HashSet;
use std::sync::Arc;

use crossbeam::channel::{Receiver, Sender, unbounded};
use rayon::{ThreadPool, ThreadPoolBuilder};

use crate::gen::noise::{NoiseGenerator, TERRAIN};
use crate::misc::pos::{ChunkPos, ChunkSubPos};
use crate::misc::util::{CHUNK_SIZE, Direction};
use crate::world::{Chunk, Grid, tile};
use crate::world::neighbor::{NeighborAware, NeighborMatrix};
use crate::world::tile::Tile;
use crate::world::wall::Wall;

mod feature;
mod gen_const;
mod noise;

#[derive(Copy, Clone)]
pub struct GenerationContext {
    noise: NoiseGenerator,
    terrain_height: u32,
    cave_transition_height: u32,
    cave_height: u32,
    hell_transition_height: u32,
    hell_lava: u32,
}

pub struct WorldGenerator {
    context: Arc<GenerationContext>,
    sender: Sender<(ChunkPos, Chunk)>,
    receiver: Receiver<(ChunkPos, Chunk)>,
    thread_pool: ThreadPool,

    queue_chunks: HashSet<ChunkPos>,
}

impl WorldGenerator {
    pub fn new(seed: u64) -> WorldGenerator {
        let (sender, receiver) = unbounded();

        Self {
            context: Arc::new(GenerationContext {
                noise: NoiseGenerator::new(seed),
                terrain_height: 50,
                cave_transition_height: 50,
                cave_height: 600,
                hell_transition_height: 50,
                hell_lava: 150,
            }),
            sender,
            receiver,
            thread_pool: ThreadPoolBuilder::new().build().unwrap(),
            queue_chunks: HashSet::new(),
        }
    }

    pub fn add_chunk(&mut self, pos: &ChunkPos) {
        if !self.queue_chunks.contains(pos) {
            let dup_pos = pos.clone();
            let context = self.context.clone();
            let sender = self.sender.clone();
            self.thread_pool.spawn(move || {
                sender.send((dup_pos, Self::gen_chunk(&context, &dup_pos))).unwrap();
            });

            self.queue_chunks.insert(*pos);
        }
    }

    pub fn generate_chunks(&mut self) -> Option<Vec<(ChunkPos, Chunk)>> {
        let result = self.receiver.try_recv();
        if result.is_err() {
            None
        } else {
            let mut out = Vec::new();
            out.push(result.unwrap());

            while let Result::Ok((pos, chunk)) = self.receiver.try_recv() {
                out.push((pos, chunk));
            }

            Some(out)
        }
    }

    fn gen_chunk(context: &Arc<GenerationContext>, pos: &ChunkPos) -> Chunk {
        let mut chunk = Chunk::new();
        //Self::generate_terrain(context, &mut chunk, pos);
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                chunk.set(&ChunkSubPos::new(x as u8, y as u8), Tile::id(tile::STONE));
            }
        }

        chunk = Self::calc_internal_neighbors::<Wall, Chunk>(chunk);
        chunk = Self::calc_internal_neighbors::<Tile, Chunk>(chunk);
        chunk
    }

    pub fn generate_terrain(context: &Arc<GenerationContext>, chunk: &mut Chunk, pos: &ChunkPos) {
        for x in 0..CHUNK_SIZE {
            let tile_x = (x as i32 + (pos.x as i32 * CHUNK_SIZE as i32)) as i32;
            let hell_floor_height_line =
                context.noise.get_2d_range(tile_x, context.hell_lava, context.hell_lava, TERRAIN);
            let hell_ceiling_height_line =
                context.hell_lava as f64 +
                    context.noise.get_2d_range(tile_x, context.hell_transition_height, context.hell_transition_height, TERRAIN);
            let cave_height_line =
                context.hell_lava as f64 +
                    context.hell_transition_height as f64 +
                    context.noise.get_2d_range(tile_x, context.cave_height, context.cave_height, TERRAIN);

            let cave_transition_height =
                context.hell_lava as f64 +
                    context.hell_transition_height as f64 +
                    context.cave_height as f64 +
                    context.noise.get_2d_range(tile_x, context.cave_transition_height, context.cave_transition_height, TERRAIN);
            let terrain_height_line =
                context.hell_lava as f64 + context.hell_transition_height as f64 + context.cave_transition_height as f64 + context.cave_height as f64 +
                    context.noise.get_2d_range(tile_x, context.terrain_height, context.terrain_height, TERRAIN);

            for y in 0..CHUNK_SIZE {
                let tile_y = (y as i32 + (pos.y as i32 * CHUNK_SIZE as i32)) as i32;
                if tile_y < hell_floor_height_line as i32 {
                    chunk.set(&ChunkSubPos::new(x as u8, y as u8), Tile::id(tile::ASH_BLOCK));
                } else if tile_y < hell_ceiling_height_line as i32 {
                    // stuff
                } else if tile_y < cave_height_line as i32 {
                    chunk.set(&ChunkSubPos::new(x as u8, y as u8), Tile::id(tile::STONE));
                } else if tile_y < cave_transition_height as i32 {
                    chunk.set(&ChunkSubPos::new(x as u8, y as u8), Tile::id(tile::STONE));
                } else if tile_y < terrain_height_line as i32 {
                    chunk.set(&ChunkSubPos::new(x as u8, y as u8), Tile::id(tile::DIRT));
                }
            }
        }
    }

    pub fn calc_internal_neighbors<N: NeighborAware, G: Grid<N>>(owner: G) -> G {
        let grid = owner.get_grid();
        // update bottom right neighbors.
        for y in 0..(CHUNK_SIZE - 1) {
            let row = &grid[y];
            let row_below = &grid[y + 1];
            for x in 0..(CHUNK_SIZE - 1) {
                let obj = &row[x];
                let obj_after = &row[x + 1];
                let obj_below = &row_below[x];
                unsafe {
                    // mutates the values!!!
                    NeighborMatrix::update_neighbor(obj, obj_after, Direction::Right);
                    NeighborMatrix::update_neighbor(obj, obj_below, Direction::Top);
                }
            }
        };

        // update right row
        for y in 0..(CHUNK_SIZE - 1) {
            let row = &grid[y];
            let row_below = &grid[y + 1];
            let obj = &row[CHUNK_SIZE - 1];
            let obj_below = &row_below[CHUNK_SIZE - 1];
            unsafe {
                // mutates the values!!!
                NeighborMatrix::update_neighbor(obj, obj_below, Direction::Top);
            }
        };

        // update bottom column
        for x in 0..(CHUNK_SIZE - 1) {
            let row = &grid[CHUNK_SIZE - 1];
            let obj = &row[x];
            let obj_after = &row[x + 1];
            unsafe {
                // mutates the values!!!
                NeighborMatrix::update_neighbor(obj, obj_after, Direction::Right);
            }
        };

        owner
    }
}