use bevy::{
    asset::{Asset, AssetId, Assets},
    ecs::{
        bundle::Bundle,
        component::{Component, Mutable},
        entity::Entity,
        query::Without,
        relationship::Relationship,
        system::{Commands, Query},
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
    DefaultProjectileBundle, ParentGlobalTransform, ParentTransform, ProjectileBundle,
    ProjectileInstance, WorldSpaceChildOf, traits::ProjectileRc,
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

    /// Obtain the transform of an external entity, must not contain a [`ProjectileInstance`].
    pub fn transform_of(&self, entity: Entity) -> Option<(&Transform, &GlobalTransform)> {
        self.tracking.get(entity).ok()
    }

    /// If [`ParentTransform`] is available, record a global space parent's transform and return it,
    /// otherwise return [`Transform::IDENTITY`].
    pub fn parent_transform(&self) -> Transform {
        self.entity_mut
            .get::<ParentTransform>()
            .map(|x| x.0)
            .unwrap_or(Transform::IDENTITY)
    }

    /// If [`ParentGlobalTransform`] is available, record a global space parent's transform and return it,
    /// otherwise return [`GlobalTransform::IDENTITY`].
    pub fn parent_global_transform(&self) -> GlobalTransform {
        self.entity_mut
            .get::<ParentGlobalTransform>()
            .map(|x| x.0)
            .unwrap_or(GlobalTransform::IDENTITY)
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
    pub fn spawn_related<R: Relationship>(&mut self, bundle: impl Bundle) {
        let entity = self.entity();
        self.commands.entity(entity).with_related::<R>(bundle);
    }
}
