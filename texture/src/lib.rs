#![allow(clippy::new_without_default)]
#![allow(clippy::field_reassign_with_default)]
mod distortion;
mod lazy;
mod noise;
mod util;
mod voronoi;
pub use ::noise as noise_rs;
use bevy::{
    asset::RenderAssetUsages,
    image::Image,
    math::{Vec2, Vec3, Vec4, Vec4Swizzles},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
pub use distortion::*;
pub use lazy::*;
pub use noise::*;
pub use voronoi::*;

#[doc(hidden)]
pub use bevy::image::ImageAddressMode;

pub trait ImageBuilder: Sized {
    /// Sample a single value at a point.
    ///
    /// Normally in `0..1` regardless of dimension, but must support all ranges
    /// since we allow input mapping and distortion.
    ///
    /// If input is colored, should return `self.sample_color(position).x`.
    fn sample(&self, position: Vec2) -> f32;

    /// Sample a color at a point.
    ///
    /// If not specified, returns `(sampled, sampled, sampled, 1)`.
    fn sample_color(&self, position: Vec2) -> Vec4 {
        let x = self.sample(position);
        Vec4::new(x, x, x, 1.)
    }

    /// Multiplies two nodes.
    fn mix(self, node: impl ImageBuilder) -> impl ImageBuilder {
        ImageMultiply(self, node)
    }

    /// Maps sampled grayscale value into a grayscale image.
    fn map_value(self, f: impl Fn(Vec2, f32) -> f32) -> impl ImageBuilder {
        NoiseMappedSampler {
            base: self,
            function: f,
        }
    }

    /// Map colors while maintaining the alpha value.
    fn map_rgb(self, f: impl Fn(Vec2, Vec3) -> Vec3) -> impl ImageBuilder {
        ColorMappedSampler {
            base: self,
            function: move |pos, vec| f(pos, vec.xyz()).extend(vec.w),
        }
    }

    /// Map colors.
    fn map_color(self, f: impl Fn(Vec2, Vec4) -> Vec4) -> impl ImageBuilder {
        ColorMappedSampler {
            base: self,
            function: f,
        }
    }

    /// Turn a grayscale image into a white image with an alpha channel.
    fn alpha_white(self) -> impl ImageBuilder {
        ColorMappedSampler {
            base: self,
            function: |_, x| Vec4::new(1., 1., 1., x.x),
        }
    }

    /// Multiplies the effective signed value of a noise.
    ///
    /// # Note
    ///
    /// This treats `0.5` as the effective `0`.
    fn amplify(self, value: f32) -> impl ImageBuilder {
        NoiseAmplify {
            noise: self,
            fac: value,
        }
    }

    /// Divides the sampled position by scale.
    fn zoom_in(self, scale: Vec2) -> impl ImageBuilder {
        ScaledInput::new(self, Vec2::ONE / scale)
    }

    /// Multiplies the sampled position by scale.
    fn zoom_out(self, scale: Vec2) -> impl ImageBuilder {
        ScaledInput::new(self, scale)
    }

    /// Distort the image with noises.
    fn distort(self, x: impl ImageBuilder, y: impl ImageBuilder) -> impl ImageBuilder {
        DistortionImage {
            base: self,
            distortion: JoinXY(x, y),
        }
    }

    /// Convert the builder to an image, with size.
    fn to_image(&self, width: usize, height: usize) -> Image {
        let mut data = vec![0; width * height * 4];
        let w = (width - 1) as f32;
        let h = (height - 1) as f32;
        let mut p = 0;
        for y in 0..height {
            for x in 0..width {
                let v = self.sample_color(Vec2::new(x as f32 / w, y as f32 / h));
                let v = (v * 255.).as_u8vec4();
                data[p] = v.x;
                data[p + 1] = v.y;
                data[p + 2] = v.z;
                data[p + 3] = v.w;
                p += 4;
            }
        }
        Image::new(
            Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::all(),
        )
    }
}
pub struct PureColorSampler(pub Vec4);

impl ImageBuilder for PureColorSampler {
    fn sample(&self, _: Vec2) -> f32 {
        self.0.x
    }

    fn sample_color(&self, _: Vec2) -> Vec4 {
        self.0
    }
}

struct ImageMultiply<A: ImageBuilder, B: ImageBuilder>(pub A, pub B);

impl<A: ImageBuilder, B: ImageBuilder> ImageBuilder for ImageMultiply<A, B> {
    fn sample(&self, position: Vec2) -> f32 {
        self.0.sample(position) * self.1.sample(position)
    }

    fn sample_color(&self, position: Vec2) -> Vec4 {
        self.0.sample_color(position) * self.1.sample_color(position)
    }
}

struct FunctionSampler<F: Fn(Vec2) -> Vec4>(F);

impl<F: Fn(Vec2) -> Vec4> ImageBuilder for FunctionSampler<F> {
    fn sample(&self, position: Vec2) -> f32 {
        (self.0)(position).x
    }

    fn sample_color(&self, position: Vec2) -> Vec4 {
        (self.0)(position)
    }
}

struct ColorMappedSampler<B: ImageBuilder, F: Fn(Vec2, Vec4) -> Vec4> {
    base: B,
    function: F,
}

impl<B: ImageBuilder, F: Fn(Vec2, Vec4) -> Vec4> ImageBuilder for ColorMappedSampler<B, F> {
    fn sample(&self, position: Vec2) -> f32 {
        self.sample_color(position).x
    }

    fn sample_color(&self, position: Vec2) -> Vec4 {
        (self.function)(position, self.base.sample_color(position))
    }
}

struct NoiseMappedSampler<B: ImageBuilder, F: Fn(Vec2, f32) -> f32> {
    base: B,
    function: F,
}

impl<B: ImageBuilder, F: Fn(Vec2, f32) -> f32> ImageBuilder for NoiseMappedSampler<B, F> {
    fn sample(&self, position: Vec2) -> f32 {
        (self.function)(position, self.base.sample(position))
    }
}

struct SampleToColorSampler<B: ImageBuilder, F: Fn(Vec2, f32) -> Vec4> {
    base: B,
    function: F,
}

impl<B: ImageBuilder, F: Fn(Vec2, f32) -> Vec4> ImageBuilder for SampleToColorSampler<B, F> {
    fn sample(&self, position: Vec2) -> f32 {
        self.sample_color(position).x
    }

    fn sample_color(&self, position: Vec2) -> Vec4 {
        (self.function)(position, self.base.sample(position))
    }
}
