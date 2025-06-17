use crate::ImageBuilder;
use bevy::math::Vec2;
use std::f32::consts::PI;

#[derive(Debug, Default)]
pub enum WaveAngle {
    #[default]
    X,
    Y,
    Angle(f32),
}

impl From<f32> for WaveAngle {
    fn from(value: f32) -> Self {
        WaveAngle::Angle(value)
    }
}

pub struct WaveImage {
    pub scale: u32,
    pub angle: WaveAngle,
    pub phase_offset: f32,
}

impl Default for WaveImage {
    fn default() -> Self {
        Self {
            scale: 5,
            angle: WaveAngle::X,
            phase_offset: 0.0,
        }
    }
}

impl ImageBuilder for WaveImage {
    fn sample(&self, position: Vec2) -> f32 {
        let x = match self.angle {
            WaveAngle::X => position.x,
            WaveAngle::Y => position.y,
            WaveAngle::Angle(angle) => {
                let (sin, cos) = angle.sin_cos();
                let len = sin.max(cos);
                position.dot(Vec2::new(cos, sin)) / len
            }
        };
        ((x * self.scale as f32 + self.phase_offset) * PI * 2.0).sin() * 0.5 + 0.5
    }
}
