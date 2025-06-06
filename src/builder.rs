use crate::{Projectile, ProjectileSpawner};

pub struct WithSpawner<A, T: ProjectileSpawner> {
    pub base: A,
    pub spawner: T,
}

impl<A: Projectile, T: ProjectileSpawner> Projectile for WithSpawner<A, T> {
    fn duration(&self) -> f32 {
        self.base.duration()
    }

    fn fac_curve(&self, fac: f32) -> f32 {
        self.base.fac_curve(fac)
    }

    fn is_expired(&self, cx: &crate::ProjectileContext) -> bool {
        self.base.is_expired(cx)
    }

    fn update(&mut self, cx: &mut crate::ProjectileContext, dt: f32) {
        self.base.update(cx, dt);
    }

    fn on_expire(&mut self, cx: &mut crate::ProjectileContext) {
        self.base.on_expire(cx);
    }

    fn apply_command(&mut self, command: &dyn std::any::Any) {
        self.base.apply_command(command);
    }

    fn as_spawner(&mut self) -> Option<&mut impl ProjectileSpawner> {
        Some(&mut self.spawner)
    }
}

impl<A: ProjectileSpawner, T: ProjectileSpawner> ProjectileSpawner for WithSpawner<A, T> {
    fn spawn_projectile(
        &mut self,
        cx: &crate::ProjectileContext,
    ) -> Option<impl crate::ProjectileBundle + use<A, T>> {
        self.base.spawn_projectile(cx)
    }

    fn space(&self) -> crate::ProjectileSpace {
        self.base.space()
    }

    fn update(&mut self, cx: &mut crate::ProjectileContext, dt: f32) {
        self.base.update(cx, dt);
    }

    fn apply_command(&mut self, command: &dyn std::any::Any) {
        self.base.apply_command(command);
    }

    fn duration(&self) -> f32 {
        self.base.duration()
    }

    fn fac_curve(&self, fac: f32) -> f32 {
        self.base.fac_curve(fac)
    }

    fn is_complete(&self, cx: &crate::ProjectileContext) -> bool {
        self.base.is_complete(cx)
    }

    fn children(
        &self,
        cx: &bevy::ecs::world::EntityMutExcept<impl bevy::ecs::bundle::Bundle>,
    ) -> impl Iterator<Item = bevy::ecs::entity::Entity> {
        self.base.children(cx)
    }

    fn extension(&mut self) -> Option<&mut impl ProjectileSpawner> {
        Some(&mut self.spawner)
    }
}
