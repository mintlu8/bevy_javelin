//! Utility for implementing particles.

use std::{
    f32::consts::PI,
    ops::{Add, AddAssign, Div, Mul, Range, Sub},
};

use bevy::{
    math::{Quat, Vec2, Vec3, Vec4},
    transform::components::Transform,
};
use fastrand::Rng;

fn lerp(a: f32, b: f32, fac: f32) -> f32 {
    a * (1.0 - fac) + b * fac
}

/// Extension traits on [`Rng`] to generate random stuff.
pub trait ProjectileRng {
    /// Create a random radian in `0..2π`.
    fn random_radian(&mut self) -> f32;

    /// Create a random 2d unit vector.
    fn random_circle(&mut self) -> Vec2;

    /// Create a random 2d vector inside a (1, 1) circle.
    fn random_in_circle(&mut self) -> Vec2;

    /// Create a random tangent vector.
    fn random_tangent(&mut self, points_to: Vec3) -> Vec3;

    /// Create a random 3d unit vector near a direction.
    fn random_cone(&mut self, points_to: Vec3, angle: f32) -> Vec3;

    /// Create a random 3d unit vector.
    fn random_sphere(&mut self) -> Vec3;

    /// Create a random [`Quat`].
    fn random_quat(&mut self) -> Quat;

    /// Create a random [`Quat`] facing a direction.
    fn random_quat_facing(&mut self, direction: Vec3) -> Quat;
}

impl ProjectileRng for Rng {
    fn random_radian(&mut self) -> f32 {
        self.f32() * (2. * PI)
    }

    fn random_circle(&mut self) -> Vec2 {
        Vec2::from_angle(self.random_radian())
    }

    fn random_in_circle(&mut self) -> Vec2 {
        let r = self.f32().sqrt();
        let (s, c) = self.random_radian().sin_cos();
        Vec2::new(r * c, r * s)
    }

    fn random_tangent(&mut self, points_to: Vec3) -> Vec3 {
        let theta = self.random_radian();
        let (sin, cos) = theta.sin_cos();
        let v = Vec3::new(sin, cos, 0.);
        Quat::from_rotation_arc(Vec3::Z, points_to).mul_vec3(v)
    }

    fn random_cone(&mut self, points_to: Vec3, angle: f32) -> Vec3 {
        let theta = self.random_radian();
        let angle = angle.cos();
        let phi = (lerp(1.0, angle, self.f32())).acos();
        let (ps, pc) = phi.sin_cos();
        let (ts, tc) = theta.sin_cos();
        Quat::from_rotation_arc(Vec3::Z, points_to).mul_vec3(Vec3::new(ps * tc, ps * ts, pc))
    }

    fn random_sphere(&mut self) -> Vec3 {
        let theta = self.random_radian();
        let phi = (self.f32() * 2. - 1.).acos();
        let (ps, pc) = phi.sin_cos();
        let (ts, tc) = theta.sin_cos();
        Vec3::new(ps * tc, ps * ts, pc)
    }

    fn random_quat(&mut self) -> Quat {
        let u1 = self.f32();
        let u2 = self.f32();
        let u3 = self.f32();
        Quat::from_array([
            (1. - u1).sqrt() * (2. * PI * u2).sin(),
            (1. - u1).sqrt() * (2. * PI * u2).cos(),
            (u1).sqrt() * (2. * PI * u3).sin(),
            (u1).sqrt() * (2. * PI * u3).cos(),
        ])
    }

    fn random_quat_facing(&mut self, facing: Vec3) -> Quat {
        Quat::from_rotation_arc(Vec3::NEG_Z, facing.normalize())
            .mul_quat(Quat::from_axis_angle(facing, self.random_radian()))
            .normalize()
    }
}

/// Place [`Transform`] on a curve while facing forward via derivatives.
///
/// This is convenient though might not be the fastest option.
pub fn transform_from_derivative(mut curve: impl FnMut(f32) -> Vec3, time: f32) -> Transform {
    const SMOL_NUM: f32 = 0.001;
    let translation = curve(time);
    let next = curve(time + SMOL_NUM);
    Transform::from_translation(translation).looking_to(next - translation, Vec3::Y)
}

/// Extension traits for performing physics on floats and vectors.
pub trait PhysicsExt: AddAssign<Self> + Mul<f32, Output = Self> + Copy {
    fn _length(&self) -> f32;

    fn move_near(&mut self, target: Self, by: f32);

    fn acceleration(&mut self, velocity: &mut Self, acceleration: Self, dt: f32) {
        *self += *velocity * dt;
        *velocity += acceleration * dt;
    }

    fn acceleration_with_drag(
        &mut self,
        velocity: &mut Self,
        acceleration: Self,
        drag: f32,
        dt: f32,
    ) {
        *self += *velocity * dt;
        let drag = velocity._length() * drag * dt;
        *velocity += *velocity * (-drag);
        *velocity += acceleration * dt;
    }

    fn apply_drag(&mut self, velocity: &mut Self, drag: f32, dt: f32) {
        *self += *velocity * dt;
        let drag = velocity._length() * drag * dt;
        *velocity += *velocity * (-drag);
    }
}

impl PhysicsExt for f32 {
    fn _length(&self) -> f32 {
        *self
    }

    fn move_near(&mut self, target: Self, by: f32) {
        *self = if *self > target {
            if *self - by > target {
                *self - by
            } else {
                target
            }
        } else if *self + by < target {
            *self + by
        } else {
            target
        }
    }
}

impl PhysicsExt for Vec2 {
    fn _length(&self) -> f32 {
        self.length()
    }

    fn move_near(&mut self, target: Self, by: f32) {
        *self = self.move_towards(target, by);
    }
}

impl PhysicsExt for Vec3 {
    fn _length(&self) -> f32 {
        self.length()
    }

    fn move_near(&mut self, target: Self, by: f32) {
        *self = self.move_towards(target, by);
    }
}

impl PhysicsExt for Vec4 {
    fn _length(&self) -> f32 {
        self.length()
    }

    fn move_near(&mut self, target: Self, by: f32) {
        *self = self.move_towards(target, by);
    }
}

/// Calculate a factor in range `from` and apply to range `to`.
pub fn map_range<A, B>(value: A, from: Range<A>, to: Range<B>) -> B
where
    A: Copy + Sub<A, Output = A> + Div<A, Output = A> + Mul<B, Output = B>,
    B: Copy + Add<B, Output = B> + Sub<B, Output = B>,
{
    (value - from.start) / (from.end - from.start) * (to.end - to.start) + to.start
}

/// A condition or action that can only be activated once from `false` to `true`.
#[derive(Debug, Default, Clone, Copy)]
pub struct ConditionOnce(bool);

impl ConditionOnce {
    pub const fn new() -> ConditionOnce {
        ConditionOnce(false)
    }

    pub fn if_then<T>(&mut self, cond: bool, then: impl FnOnce() -> T) -> Option<T> {
        if !self.0 && cond {
            self.0 = true;
            Some(then())
        } else {
            None
        }
    }

    pub fn is_activated(&self) -> bool {
        self.0
    }

    pub fn set(&mut self, condition: impl FnOnce() -> bool) -> bool {
        if !self.0 {
            self.0 = condition()
        }
        self.0
    }
}

/// A simple counter.
#[derive(Debug, Default)]
pub struct Counter(pub usize);

impl Counter {
    pub const ZERO: Counter = Counter(0);

    /// Obtain the next value and increment the counter.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> usize {
        let result = self.0;
        self.0 += 1;
        result
    }
}

/// A dynamic value getter that retains its previous value
/// if the value source is removed.
#[derive(Debug, Clone, Copy, Default)]
pub struct RetainedValue<T>(pub T);

impl<T: Copy> RetainedValue<T> {
    pub fn get(&mut self, new: Option<T>) -> T {
        if let Some(value) = new {
            self.0 = value
        }
        self.0
    }

    pub fn get_with(&mut self, new: impl FnOnce() -> Option<T>) -> T {
        if let Some(value) = new() {
            self.0 = value
        }
        self.0
    }

    pub fn current(&self) -> T {
        self.0
    }
}
