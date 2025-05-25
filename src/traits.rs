use std::{
    any::type_name,
    ops::{Deref, DerefMut},
    sync::{Arc, Weak},
};

use bevy::{ecs::component::Component, render::view::Visibility, transform::components::Transform};

use crate::{ProjectileBundle, ProjectileContext, WorldSpaceChildOf};

struct DummyProjectile;

impl ProjectileSpawner for DummyProjectile {}
impl Projectile for DummyProjectile {}

/// Local space or world space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileSpace {
    Local,
    World,
}

/// The core projectile spawner trait.
/// 
/// A [`Projectile`] can also be a spawner via implementing [`Projectile::as_spawner`].
/// 
/// If a spawner is not a projectile, use [`ProjectileInstance::spawner`] to type erase and spawn it.
/// 
/// # How to implement
/// 
/// The simplist way to implement this trait is to use [`SpawnRate`](crate::util::SpawnRate).
/// We call `SpawnRate::update` in `update_spawner` and use `SpawnRate::spawn` in `spawn_projectile`.`
#[allow(unused_variables)]
pub trait ProjectileSpawner: Send + Sync + 'static {
    /// If should spawn, returns a [`Projectile`] and its supporting components
    /// like `Mesh3d` and `MaterialMesh3d`.
    /// This is run multiple times per frame until it returns [`None`].
    ///
    /// For a leaf projectile, do not implement this function.
    ///
    /// Example pattern:
    ///
    /// ```
    /// if fac > 1.0 {
    ///     fac -= 1.0;
    ///     Some(MyProjectile { .. })
    /// } else {
    ///     None
    /// }
    /// ```
    ///
    /// Keep in mind [`ProjectileInstance`] requires [`Transform`] and [`Visibility`]
    /// so these are not required to be specified.
    ///
    /// # Returns
    ///
    /// Either a `impl Projectile` or
    /// A tuple of `(impl Projectile, impl Bundle, impl Bundle, ..)`.
    ///
    /// Additionally in the bundle slots you can use items that implement
    /// `BundleOrAsset` like [`AddMat3`](crate::loading::AddMat3), to
    /// add or load assets directly.
    fn spawn_projectile(
        &mut self,
        cx: &ProjectileContext,
    ) -> Option<impl ProjectileBundle + use<Self>> {
        None::<DummyProjectile>
    }

    /// Returns if the projectile is in local or world space.
    ///
    /// Local space uses bevy's `Children` while world space uses [`WorldSpaceChildren`](crate::WorldSpaceChildren).
    fn space(&self) -> ProjectileSpace {
        ProjectileSpace::World
    }

    /// Runs every frame to update its content.
    /// If is also a projectile, run after `update_projectile`.
    fn update_spawner(&mut self, cx: &ProjectileContext, dt: f32) {}

    /// Optional value that is used to calculate `fac` and
    /// by default sets `is_complete` once `lifetime` reaches `duration`.
    ///
    /// Keep in mind `fac` is optional and `is_complete` can be overwritten.
    ///
    /// # Note
    ///
    /// Only the first `duration` will affect the `fac` returned by [`ProjectileContext`],
    /// the implementation on [`Projectile`] takes priority.
    fn duration(&self) -> f32 {
        f32::MAX
    }

    /// Modifies `fac`, or `lifetime / duration` by an easing curve.
    ///
    /// # Note
    ///
    /// Only the first `fac_curve` will affect the `fac` returned by [`ProjectileContext`],
    /// the implementation on [`Projectile`] takes priority.
    fn fac_curve(&self, fac: f32) -> f32 {
        fac
    }

    /// Returns true if spawning is finished.
    ///
    /// If done, `update` will not be called and make this eligible for deletion.
    ///
    /// By default checks `lifetime > duration`.
    fn is_complete(&self, cx: &ProjectileContext) -> bool {
        cx.lifetime > self.duration()
    }

    /// Should be used if we want to spawn multiple types of projectiles.
    fn extension(&mut self) -> Option<&mut impl ProjectileSpawner> {
        None::<&mut DummyProjectile>
    }
}

/// The core projectile trait.
#[allow(unused_variables)]
pub trait Projectile: Send + Sync + 'static {
    /// Optional value that is used to calculate `fac` and
    /// by default sets `is_complete` once `lifetime` reaches `duration`.
    ///
    /// Keep in mind `fac` is optional and `is_complete` can be overwritten.
    fn duration(&self) -> f32 {
        f32::MAX
    }

    /// Modifies `fac`, or `lifetime / duration` by an easing curve.
    fn fac_curve(&self, fac: f32) -> f32 {
        fac
    }

    /// Returns true if projectile has expired.
    ///
    /// By default checks `lifetime > duration`.
    fn is_expired(&self, cx: &ProjectileContext) -> bool {
        cx.lifetime > self.duration()
    }

    /// Updates the projectile, will not be called if expired.
    fn update_projectile(&mut self, cx: &mut ProjectileContext, dt: f32) {}

    /// Run once when `is_expired` returns true for the first time.
    ///
    /// By default this despawns the entity, if this is not desired, overwrite this behavior.
    fn on_expire(&mut self, cx: &mut ProjectileContext) {
        cx.despawn();
    }

    /// If this projectile spawns child projectiles, add them here.
    fn as_spawner(&mut self) -> Option<&mut impl ProjectileSpawner> {
        None::<&mut DummyProjectile>
    }
}

pub trait ErasedProjectile: Send + Sync + 'static {
    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }

    /// Returns true if done.
    fn update(&mut self, cx: ProjectileContext, dt: f32) -> bool;
}

#[derive(Debug, Clone)]
pub(crate) enum ProjectileRc {
    Owned(Arc<()>),
    Released(Weak<()>),
}

impl ProjectileRc {
    pub fn new() -> Self {
        ProjectileRc::Owned(Arc::new(()))
    }

    pub fn release(&mut self) {
        match self {
            ProjectileRc::Owned(rc) => *self = ProjectileRc::Released(Arc::downgrade(rc)),
            ProjectileRc::Released(_) => (),
        }
    }

    pub fn should_drop(&mut self) -> bool {
        match self {
            ProjectileRc::Owned(_) => false,
            ProjectileRc::Released(weak) => weak.strong_count() == 0,
        }
    }
}

/// An instance of a projectile.
///
/// Requires [`Transform`] and [`Visibility`].
///
/// # Note
///
/// By default we require [`Visibility::Visible`] over [`Visibility::Inherited`],
/// this way we can disable parent projectiles without structural changes.
/// Explicitly specify [`Visibility::Inherited`] to overwrite this behavior.
#[derive(Component)]
#[require(Transform, Visibility::Visible)]
pub struct ProjectileInstance {
    pub(crate) projectile: Box<dyn ErasedProjectile>,
    pub(crate) lifetime: f32,
    /// Tracks all children, despawns if 0.
    pub(crate) rc: ProjectileRc,
    pub(crate) done: bool,
    pub(crate) root: bool,
}

impl Default for ProjectileInstance {
    fn default() -> Self {
        Self::new(DummyProjectile)
    }
}

impl ProjectileInstance {
    pub fn new(projectile: impl Projectile) -> Self {
        ProjectileInstance {
            projectile: Box::new(ErasedProjectileInst {
                projectile,
                expired: false,
            }),
            lifetime: 0.0,
            rc: ProjectileRc::new(),
            done: false,
            root: true,
        }
    }

    pub(crate) fn new_with_reference(
        projectile: impl Projectile,
        reference: &ProjectileRc,
    ) -> Self {
        ProjectileInstance {
            projectile: Box::new(ErasedProjectileInst {
                projectile,
                expired: false,
            }),
            lifetime: 0.0,
            rc: reference.clone(),
            done: false,
            root: false,
        }
    }

    pub fn spawner(projectile: impl ProjectileSpawner) -> Self {
        ProjectileInstance {
            projectile: Box::new(ErasedSpawner(projectile)),
            lifetime: 0.0,
            rc: ProjectileRc::new(),
            done: false,
            root: true,
        }
    }
}

impl Deref for ProjectileInstance {
    type Target = dyn ErasedProjectile;

    fn deref(&self) -> &Self::Target {
        self.projectile.as_ref()
    }
}

impl DerefMut for ProjectileInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.projectile.as_mut()
    }
}

struct ErasedSpawner<T>(T);

impl<T: ProjectileSpawner> ErasedProjectile for ErasedSpawner<T> {
    fn update(&mut self, mut cx: ProjectileContext, dt: f32) -> bool {
        update_spawner(&mut self.0, &mut cx, dt);
        spawner_done(&mut self.0, &cx)
    }
}

struct ErasedProjectileInst<T> {
    projectile: T,
    expired: bool,
}

impl<T: Projectile> ErasedProjectile for ErasedProjectileInst<T> {
    fn update(&mut self, mut cx: ProjectileContext, dt: f32) -> bool {
        if !self.projectile.is_expired(&cx) {
            cx.fac = self
                .projectile
                .fac_curve(cx.lifetime / self.projectile.duration());
            Projectile::update_projectile(&mut self.projectile, &mut cx, dt);
        } else if !self.expired {
            self.expired = true;
            self.projectile.on_expire(&mut cx);
        }
        if let Some(spawner) = self.projectile.as_spawner() {
            update_spawner(spawner, &mut cx, dt);
            spawner_done(spawner, &cx) && self.expired
        } else {
            self.expired
        }
    }
}

fn spawner_done<T: ProjectileSpawner>(this: &mut T, cx: &ProjectileContext) -> bool {
    this.is_complete(cx) && this.extension().is_none_or(|x| spawner_done(x, cx))
}

fn update_spawner<T: ProjectileSpawner>(this: &mut T, cx: &mut ProjectileContext, dt: f32) {
    if !this.is_complete(cx) {
        ProjectileSpawner::update_spawner(this, cx, dt);
        while let Some(projectile) = this.spawn_projectile(cx) {
            let (projectile, bundle) = projectile.into_projectile_bundle(&mut cx.resources);
            let entity = cx.entity();
            match this.space() {
                ProjectileSpace::Local => {
                    cx.commands.entity(entity).with_child((
                        ProjectileInstance::new_with_reference(projectile, cx.rc),
                        bundle,
                    ));
                }
                ProjectileSpace::World => {
                    cx.commands
                        .entity(entity)
                        .with_related::<WorldSpaceChildOf>((
                            ProjectileInstance::new_with_reference(projectile, cx.rc),
                            bundle,
                        ));
                }
            }
        }
    }

    if let Some(ext) = this.extension() {
        update_spawner(ext, cx, dt);
    }
}
