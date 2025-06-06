#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]
use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        entity::Entity,
        query::Without,
        schedule::IntoScheduleConfigs,
        system::{
            Commands, FilteredResourcesMutParamBuilder, ParamBuilder, Query, SystemParamBuilder,
        },
        world::{EntityMutExcept, FilteredResourcesMut},
    },
    time::{Time, Virtual},
    transform::components::{GlobalTransform, Transform},
};

mod builder;
mod bundle;
mod cluster;
mod control;
mod hierarchy;
pub use builder::WithSpawner;
pub mod spawning;
mod traits;
pub mod util;
pub use bundle::{BundleOrAsset, ProjectileBundle};
pub use cluster::SpawnerCluster;
use cluster::{ProjectileCommand, projectile_command_system};
pub use control::ProjectileContext;
pub use fastrand::Rng;
pub use hierarchy::*;
pub use traits::{Projectile, ProjectileInstance, ProjectileSpace, ProjectileSpawner};
pub mod loading;

type DefaultProjectileBundle = (ProjectileInstance, Transform, GlobalTransform);

pub fn projectile_update(
    mut resources: FilteredResourcesMut,
    mut commands: Commands,
    query: Query<(
        Entity,
        &'static mut ProjectileInstance,
        &'static mut Transform,
        &'static GlobalTransform,
        EntityMutExcept<'static, DefaultProjectileBundle>,
    )>,
    mut tracking: Query<
        (&'static Transform, &'static GlobalTransform),
        Without<ProjectileInstance>,
    >,
) {
    let Ok((dt, elapsed)) = resources
        .get::<Time<Virtual>>()
        .map(|x| (x.delta_secs(), x.elapsed_secs()))
    else {
        return;
    };
    // Safety: cannot access the same entity, enforced by `ProjectileContext`.
    for (entity, projectile, transform, global_transform, entity_mut) in
        unsafe { query.iter_unsafe() }
    {
        // Allow split borrow.
        let projectile = projectile.into_inner();
        if projectile.done {
            if projectile.root && projectile.rc.should_drop() {
                commands.entity(entity).despawn();
            }
            continue;
        }
        projectile.lifetime += dt;
        let cx = ProjectileContext {
            transform,
            global_transform,
            entity_mut,
            resources: resources.reborrow(),
            commands: commands.reborrow(),
            // Safety: cannot access the same entity, enforced by `ProjectileContext`.
            unsafe_other: unsafe { query.reborrow_unsafe() },
            tracking: tracking.reborrow(),
            elapsed_time: elapsed,
            lifetime: projectile.lifetime,
            rc: &projectile.rc,
            fac: 0.,
        };
        if projectile.projectile.update(cx, dt) {
            projectile.done = true;
            projectile.rc.release();
        }
    }
}

/// Plugin for [`bevy_javelin`](crate).
pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProjectileCommand>();
        let system = (
            FilteredResourcesMutParamBuilder::new(|builder| {
                builder.add_write_all();
            }),
            ParamBuilder,
            ParamBuilder,
            ParamBuilder,
        )
            .build_state(app.world_mut())
            .build_system(projectile_update);
        app.add_systems(Update, projectile_command_system);
        app.add_systems(Update, system.after(projectile_command_system));
    }
}
