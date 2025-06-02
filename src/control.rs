use bevy::{
    asset::{Asset, AssetId, Assets},
    ecs::{
        bundle::Bundle,
        change_detection::DetectChanges,
        component::{Component, Mutable},
        entity::{ContainsEntity, Entity, EntityEquivalent},
        hierarchy::ChildOf,
        query::Without,
        relationship::{Relationship, RelationshipTarget},
        system::{Command, Commands, EntityCommands, Query},
        world::{EntityMutExcept, FilteredResourcesMut, Mut},
    },
    pbr::{Material, MeshMaterial3d},
    render::{
        mesh::{Mesh, Mesh2d, Mesh3d},
        view::Visibility,
    },
    sprite::{Material2d, MeshMaterial2d},
    transform::components::{GlobalTransform, Transform},
};

use crate::{
    DefaultProjectileBundle, DetachToWorldSpaceExt, ProjectileBundle, ProjectileInstance,
    WorldSpaceChildOf, traits::ProjectileRc,
};

/// Context for projectile rendering, includes access to components, resources and
/// can query other reference entity's positions.
pub struct ProjectileContext<'w, 's> {
    pub(crate) transform: Mut<'s, Transform>,
    pub(crate) global_transform: &'s GlobalTransform,
    pub(crate) entity_mut: EntityMutExcept<'s, DefaultProjectileBundle>,
    pub(crate) resources: FilteredResourcesMut<'w, 's>,
    pub(crate) tracking:
        Query<'w, 's, (&'static Transform, &'static GlobalTransform), Without<ProjectileInstance>>,
    // Safety: cannot offer access to this entity.
    pub(crate) unsafe_other: Query<
        'w,
        's,
        (
            Entity,
            &'static mut ProjectileInstance,
            &'static mut Transform,
            &'static GlobalTransform,
            EntityMutExcept<'static, DefaultProjectileBundle>,
        ),
    >,
    pub(crate) commands: Commands<'w, 's>,
    pub(crate) rc: &'s ProjectileRc,
    pub(crate) lifetime: f32,
    pub(crate) fac: f32,
}

impl ProjectileContext<'_, '_> {
    /// Obtain the current [`Entity`].
    pub fn entity(&self) -> Entity {
        self.entity_mut.id()
    }

    /// Obtain the amount of time this projectile or spawner has stayed alive.
    pub fn lifetime(&self) -> f32 {
        self.lifetime
    }

    /// Obtain `lifetime / duration`.
    ///
    /// # Note
    ///
    /// Although normally in `0..=1`, this value is not clamped.
    pub fn fac(&self) -> f32 {
        self.fac
    }

    /// Obtain [`Transform`] of the current entity.
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    /// Obtain [`Transform`] of the current entity.
    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    /// Obtain [`GlobalTransform`] of the current entity.
    pub fn global_transform(&self) -> &GlobalTransform {
        self.global_transform
    }

    /// Obtain a mutable component on the current entity.
    pub fn component<C: Component<Mutability = Mutable>>(&mut self, f: impl FnOnce(&mut C)) {
        if let Some(mut x) = self.entity_mut.get_mut::<C>() {
            f(&mut x)
        }
    }

    /// Returns if a component is added.
    pub fn is_added<C: Component>(&self) -> bool {
        if let Some(x) = self.entity_mut.get_ref::<C>() {
            x.is_added()
        } else {
            false
        }
    }

    /// Returns true if a component is added and a condition is met.
    pub fn is_added_and<C: Component>(&self, condition: impl Fn(&C) -> bool) -> bool {
        if let Some(x) = self.entity_mut.get_ref::<C>() {
            x.is_added() && condition(&x)
        } else {
            false
        }
    }

    /// Returns if a component is changed.
    pub fn is_changed<C: Component>(&self) -> bool {
        if let Some(x) = self.entity_mut.get_ref::<C>() {
            x.is_changed()
        } else {
            false
        }
    }

    /// Returns true if a component is changed and a condition is met.
    pub fn is_changed_and<C: Component>(&self, condition: impl Fn(&C) -> bool) -> bool {
        if let Some(x) = self.entity_mut.get_ref::<C>() {
            x.is_changed() && condition(&x)
        } else {
            false
        }
    }

    /// Obtain a component on the current entity.
    pub fn get_component<C: Component<Mutability = Mutable>>(&self) -> Option<&C> {
        self.entity_mut.get::<C>()
    }

    /// Obtain a mesh.
    pub fn mesh3d(&mut self, f: impl FnOnce(&mut Mesh)) {
        self.resources
            .get_mut::<Assets<Mesh>>()
            .ok()
            .and_then(|x| {
                x.into_inner()
                    .get_mut(self.entity_mut.get::<Mesh3d>()?.id())
            })
            .map(f);
    }

    /// Obtain a mesh.
    pub fn mesh2d(&mut self, f: impl FnOnce(&mut Mesh)) {
        self.resources
            .get_mut::<Assets<Mesh>>()
            .ok()
            .and_then(|x| {
                x.into_inner()
                    .get_mut(self.entity_mut.get::<Mesh2d>()?.id())
            })
            .map(f);
    }

    /// Obtain a material.
    pub fn mat3d<M: Material>(&mut self, f: impl FnOnce(&mut M)) {
        self.resources
            .get_mut::<Assets<M>>()
            .ok()
            .and_then(|x| {
                x.into_inner()
                    .get_mut(self.entity_mut.get::<MeshMaterial3d<M>>()?.id())
            })
            .map(f);
    }

    /// Obtain a material.
    pub fn mat2d<M: Material2d>(&mut self, f: impl FnOnce(&mut M)) {
        self.resources
            .get_mut::<Assets<M>>()
            .ok()
            .and_then(|x| {
                x.into_inner()
                    .get_mut(self.entity_mut.get::<MeshMaterial2d<M>>()?.id())
            })
            .map(f);
    }

    /// Obtain an asset.
    pub fn asset<A: Asset>(&mut self, id: impl Into<AssetId<A>>, f: impl FnOnce(&mut A)) {
        self.resources
            .get_mut::<Assets<A>>()
            .ok()
            .and_then(|x| x.into_inner().get_mut(id))
            .map(f);
    }

    /// Obtain the [`Transform`] of an external entity, must not contain a [`ProjectileInstance`].
    ///
    /// If not present, returns the default value.
    pub fn transform_of(&self, entity: Entity) -> Option<Transform> {
        self.tracking.get(entity).map(|x| *x.0).ok()
    }

    /// Obtain the [`GlobalTransform`] of an external entity, must not contain a [`ProjectileInstance`].
    ///
    /// If not present, returns the default value.
    pub fn global_transform_of(&self, entity: Entity) -> Option<GlobalTransform> {
        self.tracking.get(entity).map(|x| *x.1).ok()
    }

    /// If has a parent projectile instance, return its [`Transform`].
    /// otherwise return [`Transform::IDENTITY`].
    pub fn parent_transform<T: Relationship>(&self) -> Transform {
        self.entity_mut
            .get::<T>()
            .and_then(|x| (x.get() != self.entity()).then_some(x.get()))
            .and_then(|e| self.unsafe_other.get(e).ok())
            .map(|(_, _, t, ..)| *t)
            .unwrap_or_default()
    }

    /// If has a parent projectile instance, return its [`GlobalTransform`].
    /// otherwise return [`GlobalTransform::IDENTITY`].
    pub fn parent_global_transform<T: Relationship>(&self) -> GlobalTransform {
        self.entity_mut
            .get::<T>()
            .and_then(|x| (x.get() != self.entity()).then_some(x.get()))
            .and_then(|e| self.unsafe_other.get(e).ok())
            .map(|(_, _, _, t, ..)| *t)
            .unwrap_or_default()
    }

    /// Obtain the parent entity, world or local space, and verifies not equal to itself.
    pub fn parent(&self) -> Option<Entity> {
        self.entity_mut
            .get::<ChildOf>()
            .map(|x| x.parent())
            .or_else(|| {
                self.entity_mut
                    .get::<WorldSpaceChildOf>()
                    .map(|x| x.parent())
            })
            .and_then(|e| (e != self.entity()).then_some(e))
    }

    /// Obtain a component from the parent projectile system, world or local space.
    ///
    /// Returns [`None`] if parent is not a projectile system.
    ///
    /// # Note
    ///
    /// Does not work for [`ProjectileInstance`], [`Transform`] and [`GlobalTransform`].
    pub fn get_from_parent<T: Component>(&self) -> Option<&T> {
        self.parent()
            .and_then(|e| self.unsafe_other.get(e).ok())
            .and_then(|(.., entity)| entity.get())
    }
}

impl ProjectileContext<'_, '_> {
    /// Set the entity as [`Visibility::Hidden`], as a potential way to remove a projectile on expire.
    pub fn set_invisible(&mut self) {
        if let Some(mut vis) = self.entity_mut.get_mut::<Visibility>() {
            *vis = Visibility::Hidden;
        }
    }

    /// Despawn the current entity.
    pub fn despawn(&mut self) {
        let entity = self.entity();
        self.commands.entity(entity).despawn();
    }

    /// Insert a bundle to the entity.
    pub fn insert_bundle<B: Bundle>(&mut self, bundle: B) {
        let entity = self.entity();
        self.commands.entity(entity).insert(bundle);
    }

    /// Remove a bundle from the entity, as a potential way to remove a projectile on expire.
    pub fn remove_bundle<B: Bundle>(&mut self) {
        let entity = self.entity();
        self.commands.entity(entity).remove::<B>();
    }

    /// Spawn a child projectile in world space.
    pub fn spawn_world_space(&mut self, bundle: impl ProjectileBundle) {
        let entity = self.entity();
        let (projectile, bundle) = bundle.into_projectile_bundle(&mut self.resources);
        self.commands
            .entity(entity)
            .with_related::<WorldSpaceChildOf>((
                ProjectileInstance::new_with_reference(projectile, self.rc),
                bundle,
            ));
    }

    /// Spawn a child projectile in local space.
    pub fn spawn_local_space(&mut self, bundle: impl ProjectileBundle) {
        let entity = self.entity();
        let (projectile, bundle) = bundle.into_projectile_bundle(&mut self.resources);
        self.commands.entity(entity).with_child((
            ProjectileInstance::new_with_reference(projectile, self.rc),
            bundle,
        ));
    }

    /// Spawn a unrelated projectile in the world.
    pub fn spawn_disjoint(&mut self, bundle: impl ProjectileBundle) {
        let (projectile, bundle) = bundle.into_projectile_bundle(&mut self.resources);
        self.commands
            .spawn((ProjectileInstance::new(projectile), bundle));
    }

    /// Spawn an entity in the world, bypass the projectile system.
    pub fn spawn_entity(&mut self, bundle: impl Bundle) -> Entity {
        self.commands.spawn(bundle).id()
    }

    /// Spawn a related entity, bypass the projectile system.
    pub fn spawn_related<R: Relationship>(&mut self, bundle: impl Bundle) -> Entity {
        let entity = self.entity();
        self.commands.entity(entity).with_related::<R>(bundle).id()
    }

    /// Replace local space parent with world space parent, without changing [`GlobalTransform`].
    pub fn detach_to_world_space(&mut self) {
        let entity = self.entity();
        self.commands.entity(entity).detach_to_world_space();
    }

    /// Queue a [`Command`].
    pub fn queue(&mut self, command: impl Command) {
        self.commands.queue(command);
    }

    /// Iterate over child projectiles and apply [`EntityCommands`].
    ///
    /// Filters for projectiles of type `P`.
    pub fn children<T: RelationshipTarget, P: 'static>(
        &mut self,
        mut f: impl FnMut(
            Mut<P>,
            Mut<Transform>,
            &GlobalTransform,
            EntityMutExcept<DefaultProjectileBundle>,
            EntityCommands,
        ),
    ) {
        let this = self.entity();
        if let Some(children) = self.entity_mut.get::<T>() {
            for entity in children.iter() {
                // Safety: checks entity is not this.
                if entity == this {
                    continue;
                }
                let Ok((_, projectile, transform, global, entity_mut)) =
                    self.unsafe_other.get_mut(entity)
                else {
                    continue;
                };
                let Some(projectile) = ProjectileInstance::map_mut(projectile) else {
                    continue;
                };
                let commands = self.commands.entity(entity);
                f(projectile, transform, global, entity_mut, commands);
            }
        }
    }

    /// Iterate over child projectiles and apply [`EntityCommands`].
    ///
    /// Filter for projectiles of type `P`.
    pub fn iter_children<P: 'static>(
        &mut self,
        children: impl IntoIterator<Item: EntityEquivalent>,
        mut f: impl FnMut(
            Mut<P>,
            Mut<Transform>,
            &GlobalTransform,
            EntityMutExcept<DefaultProjectileBundle>,
            EntityCommands,
        ),
    ) {
        let this = self.entity();
        for entity in children {
            let entity = entity.entity();
            // Safety: checks entity is not this.
            if entity == this {
                continue;
            }
            let Ok((_, projectile, transform, global, entity_mut)) =
                self.unsafe_other.get_mut(entity)
            else {
                continue;
            };
            let Some(projectile) = ProjectileInstance::map_mut(projectile) else {
                continue;
            };
            let commands = self.commands.entity(entity);
            f(projectile, transform, global, entity_mut, commands);
        }
    }
}
