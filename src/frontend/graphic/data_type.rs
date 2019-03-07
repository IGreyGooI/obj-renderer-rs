#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub texture: [f32; 2],
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct VertUniformBlock {
    pub projection_matrix: [[f32; 4]; 4],
    pub model_view_matrix: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 4],
    pub light_position: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FragUniformBlock {
    pub ambient_light: [f32; 4],
}
