//! Smoke is created following tutorial by Gabriel Aguiar Prod
//!
//! https://www.youtube.com/watch?v=dPJQuD93-Ks

use std::sync::OnceLock;

use bevy::{
    core_pipeline::bloom::Bloom,
    math::VectorSpace,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};
use bevy_javelin::{
    Projectile, ProjectileBundle, ProjectileContext, ProjectileInstance, ProjectilePlugin,
    ProjectileSpawner,
    loading::{AddMat3, AddMesh3, LoadMesh3},
    spawning::{ProjectileSpawning, SpawnRate},
    util::{ConditionOnce, PhysicsExt, ProjectileRng},
};
use bevy_texture_gen::{
    FbmPerlinImage, ImageBuilder, LazyImage, LoadLazyImageExt, VoronoiImage, lazy_image,
};
use fastrand::Rng;
use ramp_gen::ramp;

static SHADER: OnceLock<Handle<Shader>> = OnceLock::new();

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ProjectilePlugin)
        .insert_resource(AmbientLight {
            brightness: 800.,
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, random_movement)
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, FireBallMaterial>,
        >::default())
        .load_lazy_image(&FIREBALL_TEX)
        .load_lazy_image(&FIREBALL_NOISE)
        .load_lazy_image(&NOISE_TEX)
        .run();
}

static FIREBALL_TEX: LazyImage = lazy_image!(
    256,
    256,
    VoronoiImage::new(4).map_value(|_, x| x.powi(3)),
    Repeat,
    Repeat
);

static FIREBALL_NOISE: LazyImage = lazy_image!(
    256,
    256,
    FbmPerlinImage::new().map_value(|_, x| x.powi(3)),
    Repeat,
    Repeat
);

static NOISE_TEX: LazyImage =
    lazy_image!(256, 256, VoronoiImage::new(4).alpha_white(), Repeat, Repeat);

#[derive(Debug, Component)]
struct Target;

#[derive(Debug, Clone, TypePath, Asset, AsBindGroup)]
struct FireBallMaterial {
    #[texture(100)]
    #[sampler(101)]
    pub voronoi: Handle<Image>,
    #[texture(102)]
    #[sampler(103)]
    pub noise: Handle<Image>,
}

impl MaterialExtension for FireBallMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path("fireball.wgsl".into())
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            ..Default::default()
        },
        Transform::from_xyz(0.0, 7., 30.0).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        Bloom::NATURAL,
    ));

    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 8000.,
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(10., 10., -10.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let enemy = commands
        .spawn((
            Mesh3d(
                meshes.add(
                    Capsule3d {
                        radius: 0.5,
                        half_length: 1.0,
                    }
                    .mesh(),
                ),
            ),
            MeshMaterial3d(materials.add(StandardMaterial::from_color(Srgba::BLUE))),
            Transform::from_xyz(-10., 1.25, 0.),
            Target,
        ))
        .id();

    commands.spawn((
        ProjectileInstance::spawner(FireballSpawner {
            enemy,
            rate: SpawnRate::new(0.5).with_spawn_immediately(1),
            rng: Rng::new(),
        }),
        Transform::from_xyz(10., 1.25, 0.),
    ));
    // ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(StandardMaterial::from_color(Srgba::GREEN))),
        Transform::from_xyz(0., 0., 0.),
    ));
}

fn random_movement(
    time: Res<Time<Virtual>>,
    mut tick: Local<f32>,
    mut target: Local<Vec3>,
    mut query: Query<&mut Transform, With<Target>>,
) {
    let dt = time.delta_secs();
    *tick += dt;
    if *target == Vec3::ZERO {
        *target = [
            Vec3::new(-10., 1.25, 0.),
            Vec3::new(0., 1.25, -10.),
            Vec3::new(0., 1.25, 10.),
        ][fastrand::usize(0..3)];
    }
    if *tick > 2.0 {
        *tick = 0.0;
        *target = [
            Vec3::new(-10., 1.25, 0.),
            Vec3::new(0., 1.25, -10.),
            Vec3::new(0., 1.25, 10.),
        ][fastrand::usize(0..3)];
    }
    let t = &mut query.single_mut().unwrap().translation;
    *t = t.move_towards(*target, dt * 4.0);
}

struct FireballSpawner {
    enemy: Entity,
    rate: SpawnRate,
    rng: Rng,
}

impl ProjectileSpawner for FireballSpawner {
    fn spawn_projectile(
        &mut self,
        cx: &ProjectileContext,
    ) -> Option<impl ProjectileBundle + use<>> {
        self.rate.spawn(|| {
            (
                HomingFireball {
                    target: self.enemy,
                    hit: ConditionOnce::new(),
                    smoke_spawning: SpawnRate::new(12.0),
                    rng: self.rng.fork(),
                },
                AddMesh3(Sphere::new(0.5).mesh().into()),
                AddMat3(ExtendedMaterial {
                    base: StandardMaterial {
                        base_color: (Srgba::new(8., 4., 0., 1.)).into(),
                        unlit: true,
                        ..Default::default()
                    },
                    extension: FireBallMaterial {
                        voronoi: FIREBALL_TEX.get(),
                        noise: FIREBALL_NOISE.get(),
                    },
                }),
                *cx.transform(),
            )
        })
    }

    fn update(&mut self, _: &mut ProjectileContext, dt: f32) {
        self.rate.update(dt);
    }
}

struct HomingFireball {
    target: Entity,
    hit: ConditionOnce,
    smoke_spawning: SpawnRate,
    rng: Rng,
}

impl Projectile for HomingFireball {
    fn is_expired(&self, _: &ProjectileContext) -> bool {
        self.hit.is_activated()
    }

    fn update_projectile(&mut self, cx: &mut ProjectileContext, dt: f32) {
        let Some(transform) = cx.global_transform_of(self.target) else {
            return;
        };
        let target = transform.translation();
        cx.transform_mut().translation.move_near(target, dt * 6.);
        self.hit
            .set(|| (cx.transform().translation - target).length_squared() < 0.5);
        self.smoke_spawning.update(dt);
    }

    fn as_spawner(&mut self) -> Option<&mut impl ProjectileSpawner> {
        Some(self)
    }
}

impl ProjectileSpawner for HomingFireball {
    fn spawn_projectile(
        &mut self,
        cx: &ProjectileContext,
    ) -> Option<impl ProjectileBundle + use<>> {
        self.smoke_spawning.spawn(|| {
            (
                Smoke,
                LoadMesh3("smoke.glb#Mesh0/Primitive0"),
                AddMat3(StandardMaterial {
                    base_color: Srgba::gray(0.8).into(),
                    base_color_texture: Some(NOISE_TEX.get()),
                    alpha_mode: AlphaMode::Mask(0.3),
                    ..Default::default()
                }),
                Transform {
                    translation: cx.global_transform().translation(),
                    rotation: self.rng.random_quat(),
                    scale: Vec3::splat(0.3),
                },
            )
        })
    }
}

struct Smoke;

impl Projectile for Smoke {
    fn duration(&self) -> f32 {
        3.
    }

    fn update_projectile(&mut self, cx: &mut ProjectileContext, dt: f32) {
        let fac = cx.fac();
        cx.mat3d::<StandardMaterial>(|m| {
            let ramp = |x: f32| ramp!(clamp [0.2, Srgba::gray(0.8)], [0.4, Srgba::BLACK]);
            m.base_color = ramp(fac).into();
            m.base_color.set_alpha(1.0 - fac.clamp(0., 1.));
            m.uv_transform.translation += dt
        });
        cx.transform_mut().scale = Vec3::splat(VectorSpace::lerp(0.3, 0.1, fac));
    }
}
