use bevy::{
    asset::RenderAssetUsages,
    image::Image,
    math::{Vec2, Vec4},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::ImageBuilder;

/// An image builder that guarantees leaving the edge empty on some axises,
pub struct EdgeGappedImageBuilder<T> {
    pub x: bool,
    pub y: bool,
    pub builder: T,
}

impl<T: ImageBuilder> ImageBuilder for EdgeGappedImageBuilder<T> {
    fn sample(&self, position: Vec2) -> f32 {
        self.builder.sample(position)
    }

    fn sample_color(&self, position: Vec2) -> Vec4 {
        self.builder.sample_color(position)
    }

    fn to_image(&self, width: usize, height: usize) -> bevy::image::Image {
        let mut data = vec![0; width * height * 4];
        let w = (width - 1) as f32;
        let h = (height - 1) as f32;
        let mut p = 0;
        let (ymin, ymax) = if self.y { (1, height - 1) } else { (0, height) };
        let (xmin, xmax) = if self.x { (1, width - 1) } else { (0, width) };
        if self.y {
            p += width * 4;
        }
        for y in ymin..ymax {
            if self.x {
                p += 4
            };
            for x in xmin..xmax {
                let v = self.sample_color(Vec2::new(x as f32 / w, y as f32 / h));
                let v = (v * 255.).as_u8vec4();
                data[p] = v.x;
                data[p + 1] = v.y;
                data[p + 2] = v.z;
                data[p + 3] = v.w;
                p += 4;
            }
            if self.x {
                p += 4
            };
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
