use crate::{ImageBuilder, IntoImageBuilder};
use bevy::math::{Vec2, Vec3};
use noiz::{
    Noise, NoiseFunction, SampleableFor,
    cells::OrthoGrid,
    curves::Smoothstep,
    prelude::{
        FractalLayers, LayeredNoise, MixCellGradients, Normed, Octave, Persistence, QuickGradients,
        SNormToUNorm,
    },
    rng::NoiseRng,
};

pub struct NoiseImage<T>(pub Noise<T>);

impl<T: NoiseFunction<Vec2, Output: Into<f32> + NoiseFunction<Vec2, Output: Into<f32>>>>
    NoiseImage<T>
{
    /// Change the noise to 3d, might affect how the noise gets sampled.
    pub fn into_3d(self) -> NoiseImage3d<T> {
        NoiseImage3d(self.0, 0.)
    }

    /// Change the noise to 3d, might affect how the noise gets sampled.
    pub fn into_3d_with_z(self, z: f32) -> NoiseImage3d<T> {
        NoiseImage3d(self.0, z)
    }
}

impl<T: NoiseFunction<Vec2, Output: Into<f32>>> ImageBuilder for NoiseImage<T> {
    fn sample(&self, position: Vec2) -> f32 {
        self.0.sample(position)
    }
}

pub struct NoiseImage3d<T>(Noise<T>, f32);

impl<T: NoiseFunction<Vec3, Output: Into<f32>>> ImageBuilder for NoiseImage3d<T> {
    fn sample(&self, position: Vec2) -> f32 {
        self.0.sample(position.extend(self.1))
    }
}

/// The default blender noise node.
pub struct FbmNoiseImage {
    pub size: u32,
    pub details: u32,
    pub roughness: f32,
    pub lacunarity: f32,
    pub seed: u32,
}

impl Default for FbmNoiseImage {
    fn default() -> Self {
        Self {
            size: 5,
            details: 2,
            roughness: 0.5,
            lacunarity: 2.,
            seed: 1,
        }
    }
}

impl IntoImageBuilder for FbmNoiseImage {
    fn into_image_builder(self) -> impl ImageBuilder {
        NoiseImage(Noise {
            noise: (
                LayeredNoise::new(
                    Normed::<f32>::default(),
                    Persistence(self.roughness),
                    FractalLayers {
                        layer: Octave(MixCellGradients::<
                            OrthoGrid<i32>,
                            Smoothstep,
                            QuickGradients,
                        > {
                            cells: OrthoGrid(self.size as i32),
                            gradients: QuickGradients,
                            curve: Smoothstep,
                        }),
                        lacunarity: self.lacunarity,
                        amount: self.details.max(1),
                    },
                ),
                SNormToUNorm,
            ),
            seed: NoiseRng(self.seed),
            frequency: self.size as f32,
        })
    }
}
