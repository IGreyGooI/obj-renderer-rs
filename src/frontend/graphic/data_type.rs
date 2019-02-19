#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    position: [f32; 4],
    normal: [f32; 3],
    tangent: [f32; 3],
    texture: [f32; 2],
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct VertUniformBlock {
    projection_matrix: [[f32; 4]; 4],
    model_view_matrix: [[f32; 4]; 4],
    normal_matrix: [[f32; 3]; 3],
    light_position: [f32; 4],
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FragUniformBlock {
    ambient_light: [f32; 4],
}
