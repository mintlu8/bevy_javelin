use bevy::{
    DefaultPlugins,
    app::{App, Startup, Update},
    asset::Handle,
    core_pipeline::core_2d::Camera2d,
    ecs::{
        hierarchy::ChildOf,
        system::{Commands, Query, ResMut},
    },
    image::Image,
    input::{
        ButtonInput,
        keyboard::KeyCode,
    },
    math::Vec2,
    sprite::Sprite,
};
use bevy_asset_util::{AssetCacheLayer, CachedAssetServer};
use bevy_rectray::{
    Dimension, RectrayFrame, RectrayPlugin, RectrayWindow, SyncDimension, Transform2D,
    layout::{Container, LayoutObject, ParagraphLayout},
};
use bevy_texture_gen::{
    FbmNoiseImage, IntoImageBuilder, LazyImage, VoronoiImage, WaveAngle, WaveImage, lazy_image,
};
use noiz::lengths::MinkowskiLength;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RectrayPlugin)
        .init_resource::<AssetCacheLayer>()
        .add_systems(Startup, init)
        .add_systems(Update, margin)
        .run();
}

static BASE: LazyImage = lazy_image!(512, 512, FbmNoiseImage::default());
static WIDE: LazyImage = lazy_image!(1024, 512, FbmNoiseImage::default());
static TALL: LazyImage = lazy_image!(512, 1024, FbmNoiseImage::default());
static FBM: LazyImage = lazy_image!(
    512,
    512,
    FbmNoiseImage {
        size: 2,
        ..Default::default()
    }
);

/// Test this in fact tiles.
static VORONOI: LazyImage = lazy_image!(512, 512, VoronoiImage::new(5));
static VORONOI2: LazyImage = lazy_image!(512, 512, VoronoiImage::new(5));

static LAYERED_VORONOI: LazyImage = lazy_image!(
    512,
    512,
    VoronoiImage {
        scale: 6,
        randomness: 1.,
        seed: 4
    }
    .with_layered(2, 0.5, 1.8)
);

static MINKOWSKI: LazyImage = lazy_image!(
    512,
    512,
    VoronoiImage::new(5).with_distance_fn(MinkowskiLength(0.6))
);

static VORONOI_DISSOLVE: LazyImage =
    lazy_image!(512, 512, VoronoiImage::new(5).map_value(|_, x| x.powf(3.)));

static WAVE: LazyImage = lazy_image!(
    512,
    512,
    WaveImage {
        scale: 5,
        angle: WaveAngle::Angle(f32::to_radians(30.)),
        phase_offset: 0.0
    }
);

pub fn init(mut commands: Commands, mut assets: CachedAssetServer) {
    commands.spawn(Camera2d);
    let root = commands
        .spawn((RectrayFrame::default(), RectrayWindow))
        .id();

    let container = commands
        .spawn((
            ChildOf(root),
            Container {
                layout: LayoutObject::new(ParagraphLayout::PARAGRAPH),
                //margin: Vec2::new(5., 5.),
                ..Default::default()
            },
            Transform2D::default(),
            Dimension(Vec2::new(1024., 768.)),
        ))
        .id();

    let mut spawn = |image: Handle<Image>| {
        commands.spawn((
            ChildOf(container),
            Transform2D::default(),
            Sprite {
                image,
                custom_size: Some(Vec2::new(128., 128.)),
                ..Default::default()
            },
            SyncDimension::ToDimension,
        ));
    };

    spawn(BASE.get(&mut assets));
    spawn(WIDE.get(&mut assets));
    spawn(TALL.get(&mut assets));
    spawn(FBM.get(&mut assets));
    spawn(VORONOI.get(&mut assets));
    spawn(VORONOI2.get(&mut assets));
    spawn(LAYERED_VORONOI.get(&mut assets));
    spawn(VORONOI_DISSOLVE.get(&mut assets));
    spawn(MINKOWSKI.get(&mut assets));
    spawn(WAVE.get(&mut assets));
    spawn(WAVE.get(&mut assets));
}

fn margin(mut query: Query<&mut Container>, keyboard: ResMut<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Space) {
        for mut container in query.iter_mut() {
            if container.margin == Vec2::new(0., 0.) {
                container.margin = Vec2::new(5., 5.);
            } else {
                container.margin = Vec2::ZERO;
            }
        }
    }
}
