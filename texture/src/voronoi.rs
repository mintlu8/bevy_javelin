use bevy::math::Vec2;
use noiz::{
    Noise, SampleableFor,
    cells::{OrthoGrid, Voronoi},
    prelude::{EuclideanLength, PerCellPointDistances, WorleyLeastDistance},
    rng::NoiseRng,
};

use crate::ImageBuilder;

pub type VoronoiNoise = Noise<
    PerCellPointDistances<Voronoi<false, OrthoGrid<i32>>, EuclideanLength, WorleyLeastDistance>,
>;

pub struct VoronoiImage {
    pub noise: Noise<
        PerCellPointDistances<Voronoi<false, OrthoGrid<i32>>, EuclideanLength, WorleyLeastDistance>,
    >,
    /// If some, 3d, else 2d.
    pub z: Option<f32>,
}

impl Default for VoronoiImage {
    fn default() -> Self {
        Self::new(5)
    }
}

impl VoronoiImage {
    pub fn new(frequency: i32) -> Self {
        let mut noise = VoronoiNoise::default();
        noise.frequency = frequency as f32;
        noise.noise.cells.partitoner.0 = frequency;
        Self { noise, z: None }
    }

    pub fn new3d(frequency: i32) -> Self {
        let mut noise = VoronoiNoise::default();
        noise.frequency = frequency as f32;
        noise.noise.cells.partitoner.0 = frequency;
        Self { noise, z: Some(0.) }
    }
    pub fn new_seeded(frequency: i32, seed: u32) -> Self {
        let mut noise = VoronoiNoise::default();
        noise.frequency = frequency as f32;
        noise.noise.cells.partitoner.0 = frequency;
        noise.seed = NoiseRng(seed);
        Self { noise, z: None }
    }

    pub fn new3d_seeded(frequency: i32, seed: u32) -> Self {
        let mut noise = VoronoiNoise::default();
        noise.frequency = frequency as f32;
        noise.noise.cells.partitoner.0 = frequency;
        noise.seed = NoiseRng(seed);
        Self { noise, z: Some(0.) }
    }

    // /// Sets the distance function used by the Worley cells.
    // pub fn set_distance_function(mut self, function: impl Fn(Vec2) -> f32 + 'static) -> Self {
    //     self.noise.noise.length_mode =
    //     self
    // }

    // /// Enables or disables applying the distance from the nearest seed point
    // /// to the output value.
    // pub fn set_return_type(mut self, return_type: ReturnType) -> Self {
    //     self.noise = self.noise.set_return_type(return_type);
    //     self
    // }

    // /// Sets the frequency of the seed points.
    // pub fn set_frequency(mut self, frequency: usize) -> Self {
    //     self.noise = self.noise.set_frequency(frequency as f64);
    //     self
    // }
}

impl ImageBuilder for VoronoiImage {
    fn sample(&self, position: Vec2) -> f32 {
        if let Some(z) = self.z {
            self.noise.sample(position.extend(z))
        } else {
            self.noise.sample(position)
        }
    }
}
