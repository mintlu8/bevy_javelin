use bevy::{math::VectorSpace, prelude::*};
use bevy_javelin::{
    Projectile, ProjectileBundle, ProjectileContext, ProjectileInstance, ProjectilePlugin,
    ProjectileSpawner,
    loading::{AddMat3, AddMesh3},
    util::{PhysicsExt, ProjectileRng, SpawnRate},
};
use fastrand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ProjectilePlugin)
        .insert_resource(AmbientLight {
            brightness: 800.,
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 7., 30.0).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 8000.,
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(10., 10., -10.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn(ProjectileInstance::spawner(MySpawner {
        rate: SpawnRate::new(4.),
        rng: Rng::new(),
    }));

    // ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(StandardMaterial::from_color(Srgba::GREEN))),
        Transform::from_xyz(0., 0., 0.),
    ));
}

struct MySpawner {
    rate: SpawnRate,
    rng: Rng,
}

impl ProjectileSpawner for MySpawner {
    fn is_complete(&self, _: &ProjectileContext) -> bool {
        false
    }

    fn update_spawner(&mut self, _: &ProjectileContext, dt: f32) {
        self.rate.update(dt);
    }

    fn spawn_projectile(&mut self, _: &ProjectileContext) -> Option<impl ProjectileBundle + use<>> {
        self.rate.spawn(|| {
            (
                MyProjectile {
                    velocity: (self.rng.random_circle() * 4.).extend(10.0).xzy(),
                },
                AddMesh3(Mesh::from(Sphere::new(0.5).mesh())),
                AddMat3(StandardMaterial {
                    base_color: Color::srgb(0., 1., 1.),
                    ..Default::default()
                }),
            )
        })
    }
}

struct MyProjectile {
    velocity: Vec3,
}

impl Projectile for MyProjectile {
    fn update_projectile(&mut self, cx: &mut ProjectileContext, dt: f32) {
        cx.transform_mut().translation.acceleration(
            &mut self.velocity,
            Vec3::new(0., -9.8, 0.),
            dt,
        );
        let fac = cx.fac();
        cx.mat3d::<StandardMaterial>(|x| {
            x.base_color = Srgba::new(0., 1., 1., 1.).lerp(Srgba::RED, fac).into()
        });
    }

    fn duration(&self) -> f32 {
        2.
    }

    fn is_expired(&self, cx: &ProjectileContext) -> bool {
        cx.transform().translation.y < 0.
    }

    // do nothing.
    fn on_expire(&mut self, _: &mut ProjectileContext) {}
}
