use std::any::{Any, type_name};

use bevy::ecs::{
    entity::Entity,
    event::{Event, EventReader},
    hierarchy::{ChildOf, Children},
    system::Query,
};

use crate::{
    ProjectileContext, ProjectileInstance, ProjectileSpawner,
    traits::{ErasedProjectile, ProjectileRc},
};

#[derive(Debug, Clone)]
pub struct SpawnerCluster<T: ProjectileSpawner>(Vec<T>);

impl ProjectileInstance {
    /// Create from a list of projectile spawners, spawns each as local space children and shares projectile events.
    pub fn from_spawner_iter(iter: impl IntoIterator<Item: ProjectileSpawner>) -> Self {
        Self {
            projectile: Box::new(SpawnerCluster::from_iter(iter)),
            lifetime: 0.,
            rc: ProjectileRc::new(),
            done: false,
            root: true,
        }
    }
}

impl<T: ProjectileSpawner> FromIterator<T> for SpawnerCluster<T> {
    fn from_iter<A: IntoIterator<Item = T>>(iter: A) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: ProjectileSpawner> ErasedProjectile for SpawnerCluster<T> {
    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn get_fac(&self, _: f32) -> f32 {
        0.
    }

    fn update(&mut self, mut cx: ProjectileContext, _: f32) -> bool {
        for item in self.0.drain(..) {
            cx.spawn_related::<ChildOf>(ProjectileInstance::spawner_with_reference(item, cx.rc));
        }
        true
    }

    fn apply_command(&mut self, _: &dyn Any) -> bool {
        true
    }
}

/// An [`Event`] that applies to a single projectile.
#[derive(Debug, Event)]
pub struct ProjectileCommand(Entity, Box<dyn Any + Send + Sync>);

impl ProjectileCommand {
    pub fn new(entity: Entity, command: impl Send + Sync + 'static) -> Self {
        ProjectileCommand(entity, Box::new(command))
    }
}

pub fn projectile_command_system(
    mut reader: EventReader<ProjectileCommand>,
    mut projectiles: Query<&mut ProjectileInstance>,
    children: Query<&Children>,
) {
    for ProjectileCommand(entity, command) in reader.read() {
        apply_projectile_command(&mut projectiles, &children, *entity, command.as_ref());
    }
}

fn apply_projectile_command(
    projectiles: &mut Query<&mut ProjectileInstance>,
    children: &Query<&Children>,
    entity: Entity,
    command: &dyn Any,
) {
    if let Ok(mut projectile) = projectiles.get_mut(entity) {
        if projectile.apply_command(command) {
            if let Ok(collection) = children.get(entity) {
                for child in collection {
                    apply_projectile_command(projectiles, children, *child, command);
                }
            }
        }
    }
}
