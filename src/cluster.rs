use std::any::{Any, type_name};

use bevy::ecs::{
    entity::Entity,
    hierarchy::{ChildOf, Children},
    message::{Message, MessageReader},
    system::Query,
};

use crate::{
    Projectile, ProjectileContext, ProjectileInstance,
    traits::{ErasedProjectile, ProjectileRc},
};

/// A loosely associated group of spawners and projectiles as a single object that shares garbage collection.
#[derive(Debug, Clone)]
pub struct ProjectileCluster<T: Projectile>(Vec<T>);

impl<T: Projectile> FromIterator<T> for ProjectileInstance {
    fn from_iter<A: IntoIterator<Item = T>>(iter: A) -> Self {
        Self {
            projectile: Box::new(ProjectileCluster::from_iter(iter)),
            lifetime: 0.,
            rc: ProjectileRc::new(),
            done: false,
            root: true,
        }
    }
}

impl<T: Projectile> FromIterator<T> for ProjectileCluster<T> {
    fn from_iter<A: IntoIterator<Item = T>>(iter: A) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: Projectile> ErasedProjectile for ProjectileCluster<T> {
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
            cx.spawn_related::<ChildOf>(ProjectileInstance::new_with_reference(item, cx.rc));
        }
        true
    }

    fn apply_command(&mut self, _: &dyn Any) -> bool {
        true
    }
}

/// An [`Event`] that applies to a single projectile.
#[derive(Debug, Message)]
pub struct ProjectileCommand(Entity, Box<dyn Any + Send + Sync>);

impl ProjectileCommand {
    pub fn new(entity: Entity, command: impl Send + Sync + 'static) -> Self {
        ProjectileCommand(entity, Box::new(command))
    }
}

pub fn projectile_command_system(
    mut reader: MessageReader<ProjectileCommand>,
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
