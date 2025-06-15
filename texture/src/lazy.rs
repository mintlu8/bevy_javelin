use bevy::image::Image;
use bevy_asset_util::LazyAssetCell;

pub type LazyImage = LazyAssetCell<Image>;

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
