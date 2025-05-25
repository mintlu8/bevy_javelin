use bevy::math::Vec2;

pub trait AsVec2 {
    fn as_vec2(&self) -> Vec2;
}

impl AsVec2 for f32 {
    fn as_vec2(&self) -> Vec2 {
        Vec2::splat(*self)
    }
}

impl AsVec2 for Vec2 {
    fn as_vec2(&self) -> Vec2 {
        *self
    }
}
