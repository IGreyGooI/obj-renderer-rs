pub use super::geometry::Position3D;

pub type LuminousIntensity = f32;
pub struct PointLight {
    pub position: cgmath::Point3<f32>,
}

