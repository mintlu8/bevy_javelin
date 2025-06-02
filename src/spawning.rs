use std::ops::RangeInclusive;

use bevy::ecs::hierarchy::Children;
use fastrand::Rng;

use crate::{
    ProjectileBundle, ProjectileContext, ProjectileSpace, ProjectileSpawner, WorldSpaceChildren,
};

/// A projectile spawning rate controller.
pub trait ProjectileSpawning: Send + Sync + Sized + 'static {
    fn update(&mut self, dt: f32);

    fn try_spawn(&mut self) -> bool;

    fn finished(&self) -> bool;

    /// If should spawn, call the function.
    fn spawn<T>(&mut self, f: impl FnOnce() -> T) -> Option<T> {
        if self.try_spawn() { Some(f()) } else { None }
    }

    /// Counts how many projectiles should spawn
    /// and remove them from the spawner.
    fn spawn_count(&mut self) -> usize {
        let mut count = 0;
        while self.try_spawn() {
            count += 1;
        }
        count
    }

    /// Limit the amount of projectiles can spawn.
    fn limit(self, count: usize) -> Limit<Self> {
        Limit { base: self, count }
    }

    /// If base spawner should spawn once, spawn `x` times immediately instead.
    fn in_bursts(self, x: usize) -> RandomBursts<Self> {
        RandomBursts {
            base: self,
            range: x..=x,
            current: 0,
            rng: Rng::new(),
        }
    }

    /// If base spawner should spawn once, spawn a random amount of times immediately instead.
    fn in_random_bursts(self, min: usize, max: usize) -> RandomBursts<Self> {
        RandomBursts {
            base: self,
            range: min..=max,
            current: 0,
            rng: Rng::new(),
        }
    }

    /// Convert into a local space spawner
    fn into_spawner_local<T: ProjectileBundle, F: FnMut(&mut Rng, &ProjectileContext) -> T>(
        self,
        spawn_fn: F,
    ) -> StandardSpawner<Self, F> {
        StandardSpawner {
            spawning: self,
            spawn_fn,
            rng: Rng::new(),
            space: ProjectileSpace::Local,
        }
    }

    /// Convert into a world space spawner
    fn into_spawner_world<T: ProjectileBundle, F: FnMut(&mut Rng, &ProjectileContext) -> T>(
        self,
        spawn_fn: F,
    ) -> StandardSpawner<Self, F> {
        StandardSpawner {
            spawning: self,
            spawn_fn,
            rng: Rng::new(),
            space: ProjectileSpace::World,
        }
    }
}

/// A simple linear spawning rate that never ends.
#[derive(Debug, Clone, Copy, Default)]
pub struct SpawnRate {
    times_per_second: f32,
    meta: f32,
}

impl SpawnRate {
    pub const fn new(times_per_second: f32) -> Self {
        Self {
            times_per_second,
            meta: 0.0,
        }
    }

    pub const fn set(&mut self, times_per_second: f32) {
        self.times_per_second = times_per_second;
    }

    /// Make sure some amount of projectiles spawn immediately.
    pub const fn with_spawn_immediately(mut self, times: usize) -> Self {
        self.meta += times as f32;
        self
    }

    /// Make sure some amount of projectiles spawn immediately.
    pub fn spawn_immediately(&mut self, times: usize) {
        self.meta += times as f32;
    }
}

impl ProjectileSpawning for SpawnRate {
    fn finished(&self) -> bool {
        false
    }

    fn try_spawn(&mut self) -> bool {
        if self.meta >= 1.0 {
            self.meta -= 1.0;
            true
        } else {
            false
        }
    }

    fn spawn_count(&mut self) -> usize {
        let result = self.meta.floor();
        self.meta = self.meta.fract();
        result as usize
    }

    fn update(&mut self, dt: f32) {
        self.meta += self.times_per_second * dt;
    }
}

/// Spawn `x` projectiles once, then finish.
#[derive(Debug)]
pub struct Burst(pub usize);

impl ProjectileSpawning for Burst {
    fn update(&mut self, _: f32) {}

    fn try_spawn(&mut self) -> bool {
        if self.0 > 0 {
            self.0 -= 1;
            true
        } else {
            false
        }
    }

    fn finished(&self) -> bool {
        self.0 == 0
    }
}

/// Limits the amount of projectiles spawned.
#[derive(Debug)]
pub struct Limit<T: ProjectileSpawning> {
    pub base: T,
    pub count: usize,
}

impl<T: ProjectileSpawning> ProjectileSpawning for Limit<T> {
    fn update(&mut self, dt: f32) {
        self.base.update(dt);
    }

    fn try_spawn(&mut self) -> bool {
        if self.count > 0 && self.base.try_spawn() {
            self.count -= 1;
            return true;
        }
        false
    }

    fn finished(&self) -> bool {
        self.count == 0 || self.base.finished()
    }
}

/// Spawn projectiles in bursts.
#[derive(Debug)]
pub struct RandomBursts<T: ProjectileSpawning> {
    pub base: T,
    pub range: RangeInclusive<usize>,
    pub current: usize,
    pub rng: Rng,
}

impl<T: ProjectileSpawning> ProjectileSpawning for RandomBursts<T> {
    fn update(&mut self, dt: f32) {
        self.base.update(dt);
    }

    fn try_spawn(&mut self) -> bool {
        if self.current > 0 {
            self.current -= 1;
            true
        } else if self.base.try_spawn() {
            let spawns = self.rng.usize(self.range.clone());
            if spawns == 0 {
                false
            } else {
                self.current = spawns - 1;
                true
            }
        } else {
            false
        }
    }

    fn finished(&self) -> bool {
        self.base.finished()
    }
}

pub struct StandardSpawner<T, F> {
    pub spawning: T,
    pub spawn_fn: F,
    pub rng: Rng,
    pub space: ProjectileSpace,
}

impl<T, F> StandardSpawner<T, F> {
    /// By default [`ProjectileSpawning`] creates a random seed, this overwrites that behavior.
    pub fn seeded(mut self, seed: u64) -> Self {
        self.rng = Rng::with_seed(seed);
        self
    }
}

impl<T, F, U> ProjectileSpawner for StandardSpawner<T, F>
where
    T: ProjectileSpawning,
    F: FnMut(&mut Rng, &ProjectileContext) -> U + Send + Sync + 'static,
    U: ProjectileBundle + 'static,
{
    fn spawn_projectile(
        &mut self,
        cx: &crate::ProjectileContext,
    ) -> Option<impl ProjectileBundle + use<T, F, U>> {
        self.spawning.spawn(|| (self.spawn_fn)(&mut self.rng, cx))
    }

    fn space(&self) -> crate::ProjectileSpace {
        self.space
    }

    fn update(&mut self, _: &mut crate::ProjectileContext, dt: f32) {
        self.spawning.update(dt);
    }

    fn is_complete(&self, _: &crate::ProjectileContext) -> bool {
        self.spawning.finished()
    }

    fn children(
        &self,
        cx: &bevy::ecs::world::EntityMutExcept<impl bevy::ecs::bundle::Bundle>,
    ) -> impl Iterator<Item = bevy::ecs::entity::Entity> {
        match self.space {
            ProjectileSpace::Local => cx
                .get::<Children>()
                .map(|x| x.iter().copied())
                .unwrap_or_default(),
            ProjectileSpace::World => cx
                .get::<WorldSpaceChildren>()
                .map(|x| x.into_iter())
                .unwrap_or_default(),
        }
    }
}
