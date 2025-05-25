use std::sync::OnceLock;

use bevy::{
    app::App,
    asset::{Assets, DirectAssetAccessExt, Handle},
    ecs::world::World,
    image::Image,
};

/// A static compatible lazily initialized image.
#[derive(Debug)]
pub struct LazyImage {
    get: fn() -> Image,
    cell: OnceLock<Handle<Image>>,
}

/// Create a static compatible lazily initialized image.
///
/// Syntax:
///
/// ```rust
/// static VORONOI: LazyImage = lazy_image!(512, 512, VoronoiImage::new());
/// ```
///
/// Additionally you can specify one or two [`ImageAddressMode`](crate::ImageAddressMode)s to change how sampling works.
///
/// ```rust
/// // Both x and y are repeated.
/// static VORONOI: LazyImage = lazy_image!(
///     512, 512, VoronoiImage::new(),
///     Repeat,
/// );
///
/// // Only x is repeated
/// static VORONOI: LazyImage = lazy_image!(
///     512, 512, VoronoiImage::new(),
///     Repeat,
///     ClampToEdge,
/// );
/// ```
#[macro_export]
macro_rules! lazy_image {
    ($width: expr, $height: expr, $builder: expr $(,)?) => {
        $crate::LazyImage::new(|| $crate::ImageBuilder::to_image(&$builder, $width, $height))
    };
    ($width: expr, $height: expr, $builder: expr, $address_mode: expr $(,)?) => {
        $crate::LazyImage::new(|| {
            let mut image = $crate::ImageBuilder::to_image(&$builder, $width, $height);
            {
                use $crate::ImageAddressMode::*;
                let descriptor = image.sampler.get_or_init_descriptor();
                descriptor.address_mode_u = $address_mode;
                descriptor.address_mode_v = $address_mode;
            }
            image
        })
    };
    ($width: expr, $height: expr, $builder: expr, $address_mode_u: expr, $address_mode_v: expr $(,)?) => {
        $crate::LazyImage::new(|| {
            let mut image = $crate::ImageBuilder::to_image(&$builder, $width, $height);
            {
                use $crate::ImageAddressMode::*;
                let descriptor = image.sampler.get_or_init_descriptor();
                descriptor.address_mode_u = $address_mode_u;
                descriptor.address_mode_v = $address_mode_v;
            }
            image
        })
    };
}

impl LazyImage {
    pub const fn new(f: fn() -> Image) -> LazyImage {
        LazyImage {
            get: f,
            cell: OnceLock::new(),
        }
    }

    pub fn load(&self, world: &mut World) {
        let _ = self.cell.get_or_init(|| world.add_asset((self.get)()));
    }

    pub fn get_or_load(&self, assets: &mut Assets<Image>) -> &Handle<Image> {
        self.cell.get_or_init(|| assets.add((self.get)()))
    }

    /// # Panics
    ///
    /// If not initialized.
    pub fn get(&self) -> Handle<Image> {
        self.cell.get().unwrap().clone()
    }
}

/// Extension for loading lazy image.
pub trait LoadLazyImageExt {
    fn load_lazy_image(&mut self, image: &LazyImage) -> &mut Self;
}

impl LoadLazyImageExt for World {
    fn load_lazy_image(&mut self, image: &LazyImage) -> &mut Self {
        image.load(self);
        self
    }
}

impl LoadLazyImageExt for App {
    fn load_lazy_image(&mut self, image: &LazyImage) -> &mut Self {
        image.load(self.world_mut());
        self
    }
}
