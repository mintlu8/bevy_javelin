use std::{
    any::{Any, type_name},
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::{Arc, Weak},
};

use bevy::{
    camera::visibility::Visibility,
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        hierarchy::Children,
        world::{EntityMutExcept, Mut},
    },
    transform::components::Transform,
};

use crate::{ProjectileContext, WorldSpaceChildren, builder::ProjectileJoin};

struct DummyProjectile;

impl Projectile for DummyProjectile {}

/// Local space or world space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileSpace {
    Local,
    World,
}

/// A projectile or a spawner.
///
/// # Spawner
///
/// For spawner you should always overwrite `on_expire` since the default behavior despawns the entity,
/// which is not desired since you should want to wait for other projectiles to finish.
#[allow(unused_variables)]
pub trait Projectile: Send + Sync + 'static {
    /// Optional value that is used to calculate `fac` and
    /// by default sets `is_expired` once `lifetime` reaches `duration`.
    ///
    /// Keep in mind `fac` is optional and `is_expired` can be overwritten.
    fn duration(&self) -> f32 {
        f32::MAX
    }

    /// Modifies `fac`, or `lifetime / duration` by an easing curve.
    fn fac_curve(&self, fac: f32) -> f32 {
        fac
    }

    /// Returns true if projectile has expired.
    ///
    /// This always runs after `update`, you can rely on `update` being ran at least once.
    ///
    /// By default checks `lifetime > duration`.
    fn is_expired(&self, cx: &ProjectileContext) -> bool {
        cx.lifetime > self.duration()
    }

    /// Updates the projectile, will not be called if expired.
    ///
    /// If this is a spawner, spawn child projectiles here.
    fn update(&mut self, cx: &mut ProjectileContext, dt: f32) {}

    /// Run once when projectile is created.
    fn on_create(&mut self, cx: &mut ProjectileContext) {}

    /// Run once when `is_expired` returns true for the first time.
    ///
    /// By default this despawns the entity, if this is not desired, overwrite this behavior.
    fn on_expire(&mut self, cx: &mut ProjectileContext) {
        cx.despawn();
    }

    /// Run a dynamic command on this, returns true if valid.
    fn apply_command(&mut self, command: &dyn Any) -> bool {
        false
    }

    /// Return a list of [`Entity`] child projectiles, must be [`ProjectileInstance`]s.
    ///
    /// By default, this returns [`Children`] if found, otherwise [`WorldSpaceChildren`], otherwise `[]`,
    /// rewrite this if you need a more efficient or different algorithm.
    fn children(&self, cx: &EntityMutExcept<impl Bundle>) -> impl Iterator<Item = Entity> {
        cx.get::<Children>()
            .map(|x| x.iter().copied())
            .or_else(|| cx.get::<WorldSpaceChildren>().map(|x| x.into_iter()))
            .unwrap_or([].iter().copied())
    }

    /// Should be used if we want to spawn multiple types of projectiles.
    fn extension(&mut self) -> Option<&mut impl Projectile> {
        None::<&mut DummyProjectile>
    }

    /// Join with another projectile to share their behaviors, usually used to add a spawner to a projectile.
    fn with_extension<T: Projectile>(self, extension: T) -> ProjectileJoin<Self, T>
    where
        Self: Sized,
    {
        ProjectileJoin {
            base: self,
            extension,
        }
    }
}

pub trait ErasedProjectile: Send + Sync + 'static {
    fn type_name(&self) -> &'static str;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn get_fac(&self, lifetime: f32) -> f32;

    /// Returns true if done.
    fn update(&mut self, cx: ProjectileContext, dt: f32) -> bool;

    /// Run a dynamic command on this, returns true if propagating.
    fn apply_command(&mut self, command: &dyn Any) -> bool;
}

#[derive(Clone)]
pub(crate) enum ProjectileRc {
    Owned(Arc<()>),
    Released(Weak<()>),
}

impl Debug for ProjectileRc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Owned(x) => f
                .debug_tuple("ProjectileRc")
                .field(&Arc::strong_count(x))
                .finish(),
            Self::Released(x) => f
                .debug_tuple("ReleasedProjectileRc")
                .field(&Weak::strong_count(x))
                .finish(),
        }
    }
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

    pub fn strong_count(&self) -> usize {
        match self {
            ProjectileRc::Owned(rc) => Arc::strong_count(rc),
            ProjectileRc::Released(rc) => Weak::strong_count(rc),
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
                once: false,
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
                once: false,
            }),
            lifetime: 0.0,
            rc: reference.clone(),
            done: false,
            root: false,
        }
    }

    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.projectile.as_any().downcast_ref()
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.projectile.as_any_mut().downcast_mut()
    }

    pub fn map_mut<T: 'static>(this: Mut<Self>) -> Option<Mut<T>> {
        Mut::filter_map_unchanged(this, |x| x.projectile.as_any_mut().downcast_mut())
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

struct ErasedProjectileInst<T> {
    projectile: T,
    once: bool,
    expired: bool,
}

impl<T: Projectile> ErasedProjectile for ErasedProjectileInst<T> {
    fn update(&mut self, mut cx: ProjectileContext, dt: f32) -> bool {
        if !self.expired {
            cx.fac = self
                .projectile
                .fac_curve(cx.lifetime / self.projectile.duration());
            if !self.once {
                self.once = true;
                self.projectile.on_create(&mut cx);
            }
            update_recursive(&mut self.projectile, &mut cx, dt);
            if is_expired_recursive(&mut self.projectile, &cx) {
                self.expired = true;
                self.projectile.on_expire(&mut cx);
                true
            } else {
                false
            }
        } else {
            true
        }
    }

    fn apply_command(&mut self, command: &dyn Any) -> bool {
        apply_command_recursive(&mut self.projectile, command)
    }

    fn get_fac(&self, lifetime: f32) -> f32 {
        self.projectile
            .fac_curve(lifetime / self.projectile.duration())
    }

    fn as_any(&self) -> &dyn Any {
        &self.projectile
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        &mut self.projectile
    }

    fn type_name(&self) -> &'static str {
        type_name::<T>()
    }
}

fn is_expired_recursive<T: Projectile>(this: &mut T, cx: &ProjectileContext) -> bool {
    this.is_expired(cx) && this.extension().is_none_or(|x| is_expired_recursive(x, cx))
}

fn apply_command_recursive<T: Projectile>(this: &mut T, command: &dyn Any) -> bool {
    let mut result = false;
    result |= this.apply_command(command);
    if let Some(ext) = this.extension() {
        result |= apply_command_recursive(ext, command);
    }
    result
}

fn update_recursive<T: Projectile>(this: &mut T, cx: &mut ProjectileContext, dt: f32) {
    Projectile::update(this, cx, dt);
    if let Some(ext) = this.extension() {
        update_recursive(ext, cx, dt);
    }
}
