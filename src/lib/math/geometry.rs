use cgmath::{
    num_traits::Zero,
    Vector2,
    Vector3,
    Matrix4,
    Matrix3,
};

pub type Position = Vector2<f32>;

pub type Position3D = Vector3<f32>;

pub type PerspectiveProjectionMatrix = Matrix4<f32>;

pub type NormalPerspectiveProjectionMatrix = Matrix3<f32>;