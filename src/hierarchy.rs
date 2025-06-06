use std::{iter::Copied, slice::Iter};

use bevy::{
    ecs::{
        component::Component, entity::Entity, hierarchy::ChildOf, system::EntityCommands,
        world::EntityWorldMut,
    },
    transform::commands::BuildChildrenTransformExt,
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
pub struct WorldSpaceChildOf(pub Entity);

impl WorldSpaceChildOf {
    pub fn parent(&self) -> Entity {
        self.0
    }
}

pub trait DetachToWorldSpaceExt {
    fn detach_to_world_space(&mut self) -> &mut Self;
}

impl DetachToWorldSpaceExt for EntityWorldMut<'_> {
    fn detach_to_world_space(&mut self) -> &mut Self {
        let Some(parent) = self.get::<ChildOf>() else {
            return self;
        };
        let parent = parent.parent();
        self.remove_parent_in_place();
        self.insert(WorldSpaceChildOf(parent));
        self
    }
}

impl DetachToWorldSpaceExt for EntityCommands<'_> {
    fn detach_to_world_space(&mut self) -> &mut Self {
        self.queue(|mut x: EntityWorldMut<'_>| {
            x.detach_to_world_space();
        });
        self
    }
}
