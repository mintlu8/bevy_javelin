use crate::{FbmNoiseImage, ImageBuilder, PureColorSampler, SimpleNoise};
use bevy::math::{Vec2, Vec4, Vec4Swizzles};

pub struct DistortionImage<A, B> {
    pub base: A,
    pub distortion: B,
}

impl<A: ImageBuilder, B: ImageBuilder> ImageBuilder for DistortionImage<A, B> {
    fn sample(&self, position: Vec2) -> f32 {
        self.base.sample(
            position + (self.distortion.sample_color(position).xy() - Vec2::new(0.5, 0.5)) * 2.,
        )
    }

    fn sample_color(&self, position: Vec2) -> Vec4 {
        self.base.sample_color(
            position + (self.distortion.sample_color(position).xy() - Vec2::new(0.5, 0.5)) * 2.,
        )
    }
}

pub struct JoinXY<X, Y>(pub X, pub Y);

impl JoinXY<PureColorSampler, PureColorSampler> {
    pub fn noise<T: SimpleNoise>() -> JoinXY<FbmNoiseImage<T>, FbmNoiseImage<T>> {
        JoinXY(
            FbmNoiseImage::new_seeded(41),
            FbmNoiseImage::new_seeded(7901),
        )
    }

    pub fn noise_seeded<T: SimpleNoise>(
        seed_x: u32,
        seed_y: u32,
    ) -> JoinXY<FbmNoiseImage<T>, FbmNoiseImage<T>> {
        JoinXY(
            FbmNoiseImage::new_seeded(seed_x),
            FbmNoiseImage::new_seeded(seed_y),
        )
    }
}

impl<X: ImageBuilder, Y: ImageBuilder> ImageBuilder for JoinXY<X, Y> {
    fn sample(&self, position: Vec2) -> f32 {
        self.0.sample(position)
    }

    fn sample_color(&self, position: Vec2) -> Vec4 {
        Vec4::new(self.0.sample(position), self.1.sample(position), 0., 1.)
    }
}

/// Multiply to a noise image with origin point `0.5`.
pub struct NoiseAmplify<T> {
    pub noise: T,
    pub fac: f32,
}

impl<T: ImageBuilder> ImageBuilder for NoiseAmplify<T> {
    fn sample(&self, position: Vec2) -> f32 {
        (self.noise.sample(position) - 0.5) * self.fac + 0.5
    }
}

/// Scales the input coordinate of the sampler.
pub struct ScaledInput<T> {
    pub base: T,
    pub scale: Vec2,
}

impl<T> ScaledInput<T> {
    pub fn new(base: T, scale: Vec2) -> Self {
        ScaledInput { base, scale }
    }
}

impl<T: ImageBuilder> ImageBuilder for ScaledInput<T> {
    fn sample(&self, position: Vec2) -> f32 {
        self.base.sample(position * self.scale)
    }

    fn sample_color(&self, position: Vec2) -> Vec4 {
        self.base.sample_color(position * self.scale)
    }
}
