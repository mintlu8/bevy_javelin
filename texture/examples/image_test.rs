use bevy::{
    DefaultPlugins,
    app::{App, Startup},
    asset::Handle,
    core_pipeline::core_2d::Camera2d,
    ecs::{hierarchy::ChildOf, system::Commands},
    image::Image,
    math::Vec2,
    sprite::Sprite,
};
use bevy_rectray::{
    Dimension, RectrayFrame, RectrayPlugin, RectrayWindow, SyncDimension, Transform2D,
    layout::{Container, LayoutObject, ParagraphLayout},
};
use bevy_texture_gen::{
    FbmNoiseImage, ImageBuilder, LazyImage, LoadLazyImageExt, PerlinImage, VoronoiImage, lazy_image,
};
use noise::Perlin;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RectrayPlugin)
        .load_lazy_image(&PERLIN)
        .load_lazy_image(&WIDE)
        .load_lazy_image(&TALL)
        .load_lazy_image(&FBM)
        .load_lazy_image(&VORONOI)
        .load_lazy_image(&VORONOI3D)
        .load_lazy_image(&DISTORT_VORONOI)
        .load_lazy_image(&VORONOI_DISSOLVE)
        .add_systems(Startup, init)
        .run();
}

static PERLIN: LazyImage = lazy_image!(512, 512, PerlinImage::new());
static WIDE: LazyImage = lazy_image!(1024, 512, PerlinImage::new());
static TALL: LazyImage = lazy_image!(512, 1024, PerlinImage::new());
static FBM: LazyImage = lazy_image!(512, 512, FbmNoiseImage::<Perlin>::new());

static VORONOI: LazyImage = lazy_image!(512, 512, VoronoiImage::new());
static VORONOI3D: LazyImage = lazy_image!(512, 512, VoronoiImage::new3d_seeded(88));

static DISTORT_VORONOI: LazyImage = lazy_image!(
    512,
    512,
    VoronoiImage::new().distort(
        FbmNoiseImage::<Perlin>::new_seeded(0).amplify(0.1),
        FbmNoiseImage::<Perlin>::new_seeded(1).amplify(0.1),
    )
);

static VORONOI_DISSOLVE: LazyImage =
    lazy_image!(512, 512, VoronoiImage::new3d().map_value(|_, x| x.powf(3.)));

pub fn init(mut commands: Commands) {
    commands.spawn(Camera2d);
    let root = commands
        .spawn((RectrayFrame::default(), RectrayWindow))
        .id();

    let container = commands
        .spawn((
            ChildOf(root),
            Container {
                layout: LayoutObject::new(ParagraphLayout::PARAGRAPH),
                margin: Vec2::new(5., 5.),
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

    spawn(PERLIN.get());
    spawn(WIDE.get());
    spawn(TALL.get());
    spawn(FBM.get());
    spawn(VORONOI.get());
    spawn(VORONOI3D.get());
    spawn(DISTORT_VORONOI.get());
    spawn(VORONOI_DISSOLVE.get());
}
