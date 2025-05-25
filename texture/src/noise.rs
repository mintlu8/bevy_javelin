use crate::ImageBuilder;
use bevy::math::Vec2;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin, Seedable, Simplex, SuperSimplex};

/// Represents simple seeded noises like `Perlin` and `Simplex`.
pub trait SimpleNoise: NoiseFn<f64, 2> + Seedable + Default {}

impl<T> SimpleNoise for T where T: NoiseFn<f64, 2> + Seedable + Default {}

#[derive(Debug)]
pub struct NoiseImage<T: NoiseFn<f64, 2>>(pub T);

pub type PerlinImage = NoiseImage<Perlin>;
pub type SimpleXImage = NoiseImage<Simplex>;
pub type SuperSimpleXImage = NoiseImage<SuperSimplex>;

impl<T: SimpleNoise> NoiseImage<T> {
    pub fn new() -> Self {
        Self(T::default())
    }

    pub fn new_seeded(seed: u32) -> Self {
        Self(T::default().set_seed(seed))
    }
}

impl<T: SimpleNoise> ImageBuilder for NoiseImage<T> {
    fn sample(&self, position: Vec2) -> f32 {
        let position = position * 5.;
        self.0.get(position.as_dvec2().to_array()) as f32 * 0.5 + 0.5
    }
}

pub struct FbmNoiseImage<T: SimpleNoise>(pub Fbm<T>);

pub type FbmPerlinImage = FbmNoiseImage<Perlin>;
pub type FbmSimpleXImage = FbmNoiseImage<Simplex>;
pub type FbmSuperSimpleXImage = FbmNoiseImage<SuperSimplex>;

impl<T: SimpleNoise> FbmNoiseImage<T> {
    pub fn new() -> Self {
        FbmNoiseImage(Fbm::new(0).set_frequency(5.))
    }

    pub fn new_seeded(seed: u32) -> Self {
        FbmNoiseImage(Fbm::new(seed).set_frequency(5.))
    }

    pub fn with_parameters(mut self, f: impl FnOnce(&mut Fbm<T>)) -> Self {
        f(&mut self.0);
        self
    }
}

impl<T: SimpleNoise> ImageBuilder for FbmNoiseImage<T> {
    fn sample(&self, position: Vec2) -> f32 {
        self.0.get(position.as_dvec2().to_array()) as f32 * 0.5 + 0.5
    }
}
