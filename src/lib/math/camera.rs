use super::geometry::PerspectiveProjectionMatrix as PPM;
use super::geometry::NormalPerspectiveProjectionMatrix as NPPM;

#[derive(Debug)]
pub struct Camera {
    // transforming vertices from eye space to clip space
    pub projection: PPM,
    // transforming vertices from object space to eye space
    pub view: PPM,
    // transforming vectors from object space to eye space
    pub normal: NPPM,
}

impl Camera {
    pub fn orthographic(center: cgmath::Point3<f32>,
                        eye: cgmath::Point3<f32>) -> Camera {
        Camera {
            projection: {
                let hw = 0.5 * 2.0;
                let hh = hw / 1.0;
                let near = 0.0;
                let far = 100.0;
                cgmath::ortho(-hw, hw, -hh, hh, near, far)
            },
            view: cgmath::Matrix4::look_at(
                eye,
                center,
                cgmath::Vector3::unit_y(),
            ),
            normal: cgmath::Matrix3::look_at(
                center - eye,
                cgmath::Vector3::unit_y(),
            )
        }
    }
    
    pub fn perspective(center: cgmath::Point3<f32>, eye: cgmath::Point3<f32>) -> Camera {
        Camera {
            projection: {
                let fovy = cgmath::Deg { 0: 90.0 };
                let aspect = 16.0 / 9.0;
                let near = 1.0;
                let far = 20.0;
                cgmath::perspective(fovy, aspect, near, far)
            },
            view: cgmath::Matrix4::look_at(
                eye,
                center,
                cgmath::Vector3::unit_y(),
            ),
            normal: cgmath::Matrix3::look_at(
                center - eye,
                cgmath::Vector3::unit_y(),
            )
        }
    }
}