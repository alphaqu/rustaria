use noise::{NoiseFn, Perlin, Seedable};

pub const TERRAIN: NoiseLayer = NoiseLayer { octaves: 16, scale: 512.0 };
pub const CAVE: NoiseLayer = NoiseLayer { octaves: 16, scale: 32.0 };
pub const BIOME: NoiseLayer = NoiseLayer { octaves: 1, scale: 256.0 };
pub const SUB_BIOME: NoiseLayer = NoiseLayer { octaves: 1, scale: 128.0 };
pub const STRUCTURE: NoiseLayer = NoiseLayer { octaves: 1, scale: 64.0 };

pub struct NoiseLayer {
    octaves: u8,
    scale: f64,
}

#[derive(Copy, Clone)]
pub struct NoiseGenerator {
    perlin: Perlin,
}

impl NoiseGenerator {
    pub fn new(seed: u64) -> NoiseGenerator {
        let perlin = Perlin::new();
        perlin.set_seed(seed as u32);
        Self {
            perlin,
        }
    }

    pub fn get_1d_range(&self, x: i32, scale: u32, layer: NoiseLayer) -> f64 {
        self.get_1d(x, layer) * scale as f64
    }

    pub fn get_2d_range(&self, x: i32, y: u32, scale: u32, layer: NoiseLayer) -> f64 {
       // self.get_2d(x, y, layer) * scale as f64
        0.5 * scale as f64
    }

    pub fn get_3d_range(&self, x: i32, y: u32, z: u32, scale: u32, layer: NoiseLayer) -> f64 {
        self.get_3d(x, y, z, layer) * scale as f64
    }

    pub fn get_1d(&self, x: i32, layer: NoiseLayer) -> f64 {
        let rng_x = x as f64 / layer.scale;
        let mut out = 0f64;
        for octave in 1..=layer.octaves {
            out += self.perlin.get([rng_x * octave as f64, 0.0])
        };

        out / layer.octaves as f64
    }

    pub fn get_2d(&self, x: i32, y: u32, layer: NoiseLayer) -> f64 {
        let rng_x = x as f64 / layer.scale;
        let rng_y = y as f64 / layer.scale;
        let mut out = 0f64;

        for octave in 1..=layer.octaves {
            out += self.perlin.get([rng_x * octave as f64, rng_y * octave as f64]);
        };

        out / layer.octaves as f64
    }

    pub fn get_3d(&self, x: i32, y: u32, z: u32, layer: NoiseLayer) -> f64 {
        let rng_x = x as f64 / layer.scale;
        let rng_y = y as f64 / layer.scale;
        let rng_z = z as f64 / layer.scale;
        let mut out = 0f64;
        for octave in 1..=layer.octaves {
            out += self.perlin.get([rng_x * octave as f64, rng_y * octave as f64, rng_z * octave as f64])
        };

        out / layer.octaves as f64
    }
}