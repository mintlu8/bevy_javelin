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

mod bundle;
mod control;
mod hierarchy;
mod traits;
pub mod util;
pub use bundle::{BundleOrAsset, ProjectileBundle};
pub use control::ProjectileContext;
pub use fastrand::Rng;
pub use hierarchy::*;
pub use traits::{Projectile, ProjectileInstance, ProjectileSpace, ProjectileSpawner};
pub mod loading;

type DefaultProjectileBundle = (ProjectileInstance, Transform, GlobalTransform);

pub fn projectile_update(
    mut resources: FilteredResourcesMut,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut ProjectileInstance,
        &mut Transform,
        &GlobalTransform,
        EntityMutExcept<DefaultProjectileBundle>,
    )>,
    mut tracking: Query<
        (&'static Transform, &'static GlobalTransform),
        Without<ProjectileInstance>,
    >,
) {
    let Ok(dt) = resources.get::<Time<Virtual>>().map(|x| x.delta_secs()) else {
        return;
    };
    for (entity, projectile, transform, global_transform, entity_mut) in query.iter_mut() {
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
            tracking: tracking.reborrow(),
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
        app.add_systems(Update, record_transforms);
        app.add_systems(Update, system.after(record_transforms));
    }
}
