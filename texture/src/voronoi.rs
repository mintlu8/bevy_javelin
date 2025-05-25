use bevy::math::Vec2;
use noise::{Worley, core::worley::ReturnType};
use noiz::{
    Noise, SampleableFor,
    cells::{OrthoGrid, Voronoi},
    prelude::{EuclideanLength, PerCellPointDistances, WorleyLeastDistance},
};

use crate::ImageBuilder;

pub struct VoronoiImage {
    pub noise: Worley,
    /// If some, 3d, else 2d.
    pub z: Option<f32>,
}

impl VoronoiImage {
    pub fn new() -> Self {
        Self {
            noise: Worley::new(0)
                .set_frequency(5.)
                .set_return_type(ReturnType::Distance),
            z: None,
        }
    }

    pub fn new3d() -> Self {
        Self {
            noise: Worley::new(0)
                .set_frequency(5.)
                .set_return_type(ReturnType::Distance),
            z: Some(0.),
        }
    }

    pub fn new_seeded(seed: u32) -> Self {
        Self {
            noise: Worley::new(seed)
                .set_frequency(5.)
                .set_return_type(ReturnType::Distance),
            z: None,
        }
    }

    pub fn new3d_seeded(seed: u32) -> Self {
        Self {
            noise: Worley::new(seed)
                .set_frequency(5.)
                .set_return_type(ReturnType::Distance),
            z: Some(0.),
        }
    }

    pub fn with_parameters(mut self, f: impl FnOnce(&mut Worley)) -> Self {
        f(&mut self.noise);
        self
    }

    /// Sets the distance function used by the Worley cells.
    pub fn set_distance_function(mut self, function: impl Fn(Vec2) -> f32 + 'static) -> Self {
        self.noise = self.noise.set_distance_function(move |x, y| {
            function(Vec2::new(x[0] as f32, y[0] as f32)) as f64
        });
        self
    }

    /// Enables or disables applying the distance from the nearest seed point
    /// to the output value.
    pub fn set_return_type(mut self, return_type: ReturnType) -> Self {
        self.noise = self.noise.set_return_type(return_type);
        self
    }

    /// Sets the frequency of the seed points.
    pub fn set_frequency(mut self, frequency: f32) -> Self {
        self.noise = self.noise.set_frequency(frequency as f64);
        self
    }
}

impl ImageBuilder for VoronoiImage {
    fn sample(&self, position: Vec2) -> f32 {
        let mut noise = Noise::<
            PerCellPointDistances<
                Voronoi<false, OrthoGrid<i32>>,
                EuclideanLength,
                WorleyLeastDistance,
            >,
        >::default();
        noise.frequency = 5.0;
        noise.noise.cells.randomness = 1.0;
        noise.noise.cells.partitoner.0 = 5;
        noise.sample(position)
    }
}
