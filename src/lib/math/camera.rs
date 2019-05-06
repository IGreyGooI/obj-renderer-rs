use cgmath::{Matrix, Matrix4, Point3, Transform, Zero};

#[derive(Debug)]
pub struct Camera {
    // transforming vertices from eye space to clip space
    pub projection: Matrix4<f32>,
    // transforming vertices from object space to eye space
    pub view: Matrix4<f32>,
    // transforming vectors from object space to eye space
    pub normal: Matrix4<f32>,
}

impl Camera {
    pub fn orthographic(center: Point3<f32>,
                        eye: Point3<f32>) -> Camera {
        let projection = {
            let hw = 0.5 * 2.0;
            let hh = hw / 1.0;
            let near = 0.0;
            let far = 100.0;
            cgmath::ortho(-hw, hw, -hh, hh, near, far)
        };
        
        let view = cgmath::Matrix4::look_at(
            eye,
            center,
            cgmath::Vector3::unit_y(),
        );
        
        let normal = {
            let view_inverse = view.inverse_transform();
            
            match view_inverse {
                Some(view_inverse) => {
                    view_inverse.transpose()
                }
                None => {
                    Matrix4::one()
                }
            }
        };
        Camera {
            projection,
            view,
            normal,
        }
    }
    
    pub fn perspective(center: cgmath::Point3<f32>, eye: cgmath::Point3<f32>) -> Camera {
        let projection = {
            let fovy = cgmath::Deg { 0: 70.0 };
            let aspect = 16.0 / 9.0;
            let near = 0.1;
            let far = 100.0;
            cgmath::perspective(fovy, aspect, near, far)
        };
        
        let view = cgmath::Matrix4::look_at(
            eye,
            center,
            cgmath::Vector3::zero() - cgmath::Vector3::unit_y(),
        );
        
        let normal = {
            let view_inverse = view.inverse_transform();
            
            match view_inverse {
                Some(view_inverse) => {
                    view_inverse.transpose()
                }
                None => {
                    Matrix4::one()
                }
            }
        };
        Camera {
            projection,
            view,
            normal,
        }
    }
}