use std::{
    cell::RefCell,
    rc::Rc,
};
use std::borrow::Borrow;
use std::iter;

use crate::{
    lib::{
        resource::{
            gfs::GemFileSystem,
            ReadFile,
        }
    }
};
use crate::frontend::graphic::data_type::*;

use super::{
    descriptor::DescriptorState,
    device::DeviceState,
    prelude::*,
    render_pass::RenderPassState,
    shader_module::ShaderModuleState,
};

pub struct ObjectPso {
    device_state: Rc<RefCell<DeviceState>>,
    pub pipeline: Option<<B as TB>::GraphicsPipeline>,
    pub pipeline_layout: Option<<B as TB>::PipelineLayout>,
    pub pipeline_cache: Option<<B as TB>::PipelineCache>,
}

impl ObjectPso {
    pub fn new(
        device_state: Rc<RefCell<DeviceState>>,
        render_pass: &<B as TB>::RenderPass,
        descriptor_states: Vec<&DescriptorState>,
        gfs: &mut GemFileSystem<u8>,
    ) -> Self
    {
        let descriptor_set_layouts: Vec<&<B as TB>::DescriptorSetLayout> =
            descriptor_states.into_iter().filter(|descriptor_state|
                descriptor_state.descriptor_set_layout.is_some()).map(
                |descriptor_state| {
                    descriptor_state.descriptor_set_layout.as_ref().unwrap()
                }).collect();
        
        
        let pipeline_layout = unsafe {
            let device = &device_state.borrow_mut().device;
            device.create_pipeline_layout(
                descriptor_set_layouts,
                &[],
            )
        }.unwrap();
    
        let vertex_shader_module = {
            let spirv = gfs
                .load("shaders/gen/object.vert.spv".to_string())
                .expect("Cannot load shader");
            ShaderModuleState::new(device_state.clone(), spirv)
        };
    
        let fragment_shader_module = {
            let spirv = gfs
                .load("shaders/gen/object.frag.spv".to_string())
                .expect("Cannot load shader");
            ShaderModuleState::new(device_state.clone(), spirv)
        };
        
        let pipeline = unsafe {
            let device = &device_state.borrow_mut().device;
            
            let vs_entry = EntryPoint {
                entry: "main",
                module: vertex_shader_module.module.as_ref().unwrap(),
                specialization: Default::default(),
            };
            
            let fs_entry = EntryPoint {
                entry: "main",
                module: fragment_shader_module.module.as_ref().unwrap(),
                specialization: Default::default(),
            };
            
            let shader_set = GraphicsShaderSet {
                vertex: vs_entry,
                hull: None,
                domain: None,
                geometry: None,
                fragment: Some(fs_entry),
            };
            
            let subpass = Subpass {
                index: 0,
                main_pass: render_pass,
            };
            
            let mut pipeline_desc = GraphicsPipelineDesc::new(
                shader_set,
                Primitive::TriangleList,
                Rasterizer::FILL,
                &pipeline_layout,
                subpass,
            );
    
            // what does Blending do?
            pipeline_desc
                .blender
                .targets
                .push(ColorBlendDesc {
                    0: ColorMask::ALL,
                    1: BlendState::ALPHA,
                });
    
            pipeline_desc.vertex_buffers.push(
                VertexBufferDesc {
                    binding: 0,
                    stride: std::mem::size_of::<Vertex>() as u32,
                    rate: VertexInputRate::Vertex,
                }
            );
            
            pipeline_desc.attributes.push(
                AttributeDesc {
                    location: 0,
                    binding: 0,
                    element: Element {
                        format: Format::Rgb32Sfloat,
                        offset: 0,
                    },
                }
            );
            pipeline_desc.attributes.push(
                AttributeDesc {
                    location: 1,
                    binding: 0,
                    element: Element {
                        format: Format::Rgb32Sfloat,
                        offset: 12,
                    },
                }
            );
            pipeline_desc.attributes.push(
                AttributeDesc {
                    location: 2,
                    binding: 0,
                    element: Element {
                        format: Format::Rgb32Sfloat,
                        offset: 24,
                    },
                }
            );
            pipeline_desc.attributes.push(
                AttributeDesc {
                    location: 3,
                    binding: 0,
                    element: Element {
                        format: Format::Rg32Sfloat,
                        offset: 36,
                    },
                }
            );
    
    
            device.create_graphics_pipeline(&pipeline_desc, None)
        }.unwrap();
        
        ObjectPso {
            device_state,
            pipeline: Some(pipeline),
            pipeline_layout: Some(pipeline_layout),
            pipeline_cache: None,
        }
    }
}

impl Drop for ObjectPso {
    fn drop(&mut self) {
        unsafe {
            let device = &self.device_state.borrow_mut().device;
            if let Some(pipeline) = self.pipeline.take() {
                device.destroy_graphics_pipeline(pipeline);
            }
            
            if let Some(pipeline_layout) = self.pipeline_layout.take() {
                device.destroy_pipeline_layout(pipeline_layout);
            }
            if let Some(pipeline_cache) = self.pipeline_cache.take() {
                device.destroy_pipeline_cache(pipeline_cache);
            }
        }
    }
}
