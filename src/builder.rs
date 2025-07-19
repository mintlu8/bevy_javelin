use crate::Projectile;

pub struct ProjectileJoin<A, T: Projectile> {
    pub base: A,
    pub extension: T,
}

impl<A: Projectile, T: Projectile> Projectile for ProjectileJoin<A, T> {
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

    fn apply_command(&mut self, command: &dyn std::any::Any) -> bool {
        self.base.apply_command(command)
    }

    fn extension(&mut self) -> Option<&mut impl Projectile> {
        Some(&mut self.extension)
    }
}
