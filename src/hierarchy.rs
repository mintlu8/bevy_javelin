use std::{iter::Copied, slice::Iter};

use bevy::{
    ecs::{component::Component, entity::Entity, system::Query},
    transform::components::{GlobalTransform, Transform},
};

/// Alternative children that does not inherit transform.
#[derive(Debug, Component)]
#[relationship_target(relationship = WorldSpaceChildOf)]
pub struct WorldSpaceChildren(Vec<Entity>);

impl<'t> IntoIterator for &'t WorldSpaceChildren {
    type Item = Entity;

    type IntoIter = Copied<Iter<'t, Entity>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

/// Alternative children that does not inherit transform.
#[derive(Debug, Component)]
#[relationship(relationship_target = WorldSpaceChildren)]
pub struct WorldSpaceChildOf(Entity);

/// Record world space parent's [`Transform`].
#[derive(Debug, Clone, Copy, Component)]
pub struct ParentTransform(pub Transform);

/// Record world space parent's [`GlobalTransform`].
#[derive(Debug, Clone, Copy, Component)]
pub struct ParentGlobalTransform(pub GlobalTransform);

pub fn record_transforms(
    mut transform: Query<(&WorldSpaceChildOf, &mut ParentTransform)>,
    mut global: Query<(&WorldSpaceChildOf, &mut ParentGlobalTransform)>,
    query: Query<(&Transform, &GlobalTransform)>,
) {
    for (parent, mut record) in &mut transform {
        if let Ok((transform, _)) = query.get(parent.0) {
            record.0 = *transform;
        }
    }
    for (parent, mut record) in &mut global {
        if let Ok((_, global)) = query.get(parent.0) {
            record.0 = *global;
        }
    }
}
