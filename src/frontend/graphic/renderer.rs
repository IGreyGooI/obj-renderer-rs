use std::{
    cell::RefCell,
    iter,
    rc::Rc,
};
use std::io::BufReader;
use std::io::Cursor;

use ::image::{ImageBuffer, ImageFormat, load, Pixel, Rgba};
use backend;
use gfx_hal::buffer::IndexBufferView;
use gfx_hal::IndexType;
use glsl_to_spirv::ShaderType;
use obj::{
    IndexTuple,
    Obj,
};

use crate::{
    frontend::{
        graphic::{
            data_type::*,
            hal::{
                render_pass::RenderPassState,
                pipeline::ObjectPso,
                swapchain::{SwapchainState,
                            FrameBufferState},
                device::DeviceState,
                prelude::*,
                descriptor::{
                    DescriptorState,
                    DescriptorPoolState,
                },
                buffer::BufferState,
            },
        }
    },
    lib::resource::gfs::GemFileSystem,
};
use crate::frontend::graphic::hal::adapter::AdapterState;
use crate::frontend::graphic::hal::image::SampledImageState;
use crate::lib::math::camera::Camera;
use crate::lib::math::light::PointLight;
use crate::lib::resource::ReadFile;

use super::constants::*;
use super::window::WindowState;

pub struct RendererState {
    // Rendering Global Variables:
    pub viewport: Viewport,
    // adapter state in use for this Renderer
    pub adapter_state: AdapterState,
    // flag
    pub rebuild_swapchain: bool,
    
    // The Following are the ones implemented Drop
    // since they own part of memory on device
    // and dropping them need to call device.destory_*
    // Thus the order of dropping matters,
    // which is reflected as the order of members here!
    vertex_buffer: BufferState<Vertex>,
    vert_uniform_buffer: BufferState<VertUniformBlock>,
    //indices_buffer: BufferState<u32>,
    
    vert_uniform_descriptor_state: DescriptorState,
    normal_descriptor_state: DescriptorState,
    diffuse_descriptor_state: DescriptorState,
    specular_descriptor_state: DescriptorState,
    
    normal_descriptor_pool_state: DescriptorPoolState,
    diffuse_descriptor_pool_state: DescriptorPoolState,
    specular_descriptor_pool_state: DescriptorPoolState,
    vert_uniform_descriptor_pool_state: DescriptorPoolState,
    
    normal_image_state: SampledImageState,
    diffuse_image_state: SampledImageState,
    specular_image_state: SampledImageState,
    
    gfs: GemFileSystem<u8>,
    object_pso: ObjectPso,
    render_pass_state: RenderPassState,
    swapchain_state: Option<SwapchainState>,
    frame_buffer_state: FrameBufferState,
    device_state: Rc<RefCell<DeviceState>>,
    // Instance should be drop
    // after there is no struct with device memory left
    instance: backend::Instance,
    surface: <B as TB>::Surface,
}

impl RendererState {
    pub fn new(
        window_state: &WindowState,
        render_size: Extent2D,
    ) -> RendererState {
        let window = &window_state.window;
        let instance = backend::Instance::create(INSTANCE_NAME, 1);
        let mut surface = instance.create_surface(&window);
        
        let adapter_state = AdapterState::new(&mut instance.enumerate_adapters());
    
        let device_state = Rc::new(
            RefCell::new(
                DeviceState::new(&adapter_state, &surface)
            )
        );
        
        let mut gfs = GemFileSystem::new(
            &concat!(env!("CARGO_MANIFEST_DIR"), "\\res")
        );
    
        let mut swapchain_state = SwapchainState::new(
            device_state.clone(),
            &adapter_state,
            &mut surface,
            render_size,
        );
    
        let render_pass_state = RenderPassState::new(
            device_state.clone(),
            &swapchain_state,
        );
    
        let frame_buffer_state = FrameBufferState::new(
            device_state.clone(),
            &adapter_state,
            &render_pass_state,
            &mut swapchain_state,
        );
    
        let mut vert_uniform_descriptor_pool_state = DescriptorPoolState::new(
            device_state.clone(),
            &[
                DescriptorRangeDesc {
                    ty: DescriptorType::UniformBuffer,
                    count: 1,
                }
            ],
        );
        let mut diffuse_descriptor_pool_state = DescriptorPoolState::new(
            device_state.clone(),
            &[
                DescriptorRangeDesc {
                    ty: DescriptorType::SampledImage,
                    count: 1,
                },
                DescriptorRangeDesc {
                    ty: DescriptorType::Sampler,
                    count: 1,
                }
            ],
        );
        let mut normal_descriptor_pool_state = DescriptorPoolState::new(
            device_state.clone(),
            &[
                DescriptorRangeDesc {
                    ty: DescriptorType::SampledImage,
                    count: 1,
                },
                DescriptorRangeDesc {
                    ty: DescriptorType::Sampler,
                    count: 1,
                }
            ],
        );
        let mut specular_descriptor_pool_state = DescriptorPoolState::new(
            device_state.clone(),
            &[
                DescriptorRangeDesc {
                    ty: DescriptorType::SampledImage,
                    count: 1,
                },
                DescriptorRangeDesc {
                    ty: DescriptorType::Sampler,
                    count: 1,
                }
            ],
        );
        let mut vert_uniform_descriptor_state = DescriptorState::new(
            device_state.clone(),
            &[
                DescriptorSetLayoutBinding {
                    binding: 1,
                    ty: DescriptorType::UniformBuffer,
                    // the following field cannot be 0 or it will panic when allocatting memory
                    // for the DescriptorSet
                    count: 1,
                    stage_flags: ShaderStageFlags::VERTEX,
                    immutable_samplers: false,
                },
            ],
            &[],
        );
        let mut normal_descriptor_state = DescriptorState::new(
            device_state.clone(),
            &[
                DescriptorSetLayoutBinding {
                    binding: 0,
                    ty: DescriptorType::SampledImage,
                    count: 1,
                    stage_flags: ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                },
                DescriptorSetLayoutBinding {
                    binding: 1,
                    ty: DescriptorType::Sampler,
                    count: 1,
                    stage_flags: ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                }
            ],
            &[],
        );
        let mut diffuse_descriptor_state = DescriptorState::new(
            device_state.clone(),
            &[
                DescriptorSetLayoutBinding {
                    binding: 0,
                    ty: DescriptorType::SampledImage,
                    count: 1,
                    stage_flags: ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                },
                DescriptorSetLayoutBinding {
                    binding: 1,
                    ty: DescriptorType::Sampler,
                    count: 1,
                    stage_flags: ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                }
            ],
            &[],
        );
        let mut specular_descriptor_state = DescriptorState::new(
            device_state.clone(),
            &[
                DescriptorSetLayoutBinding {
                    binding: 0,
                    ty: DescriptorType::SampledImage,
                    count: 1,
                    stage_flags: ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                },
                DescriptorSetLayoutBinding {
                    binding: 1,
                    ty: DescriptorType::Sampler,
                    count: 1,
                    stage_flags: ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                }
            ],
            &[],
        );
    
        vert_uniform_descriptor_state.allocate_descriptor_set(
            &mut vert_uniform_descriptor_pool_state);
        normal_descriptor_state.allocate_descriptor_set(
            &mut normal_descriptor_pool_state);
        diffuse_descriptor_state.allocate_descriptor_set(
            &mut diffuse_descriptor_pool_state);
        specular_descriptor_state.allocate_descriptor_set(
            &mut specular_descriptor_pool_state);
        
        let viewport = Viewport {
            rect: Rect {
                x: 0,
                y: 0,
                w: render_size.width as i16,
                h: render_size.height as i16,
            },
            depth: 0.0..1.0,
        };
        
        let model_file = gfs
            .load("models/Chest.obj".to_string())
            .unwrap();
        let chest_obj: Obj<'_, Vec<IndexTuple>> = Obj::load(
            model_file).unwrap();
    
        // here is another way to load it by using Cursor
        //let model_file = gfs.load("models/Chest.obj".to_string()).unwrap();
        //let object_sec: Obj<Vec<IndexTuple>> = Obj::load_buf(&mut Cursor::new(model_file)).unwrap();
    
        let chest_diffuse = gfs
            .load("models/Chest-diffuse.tga".to_string())
            .unwrap();
        let chest_normal = gfs
            .load("models/Chest-normal.tga".to_string())
            .unwrap();
        let chest_specular = gfs
            .load("models/Chest-specular.tga".to_string())
            .unwrap();
    
        let diffuse_image =
            load(
                Cursor::new(chest_diffuse),
                ImageFormat::TGA)
                .unwrap()
                .to_rgba();
        let normal_image =
            load(
                Cursor::new(chest_normal),
                ImageFormat::TGA)
                .unwrap()
                .to_rgba();
        let specular_image =
            load(
                Cursor::new(chest_specular),
                ImageFormat::TGA)
                .unwrap()
                .to_rgba();
    
        let mut diffuse_image_state = SampledImageState::new(
            device_state.clone(),
            &adapter_state,
            diffuse_image,
            buffer::Usage::TRANSFER_SRC,
        );
    
        let mut normal_image_state = SampledImageState::new(
            device_state.clone(),
            &adapter_state,
            normal_image,
            buffer::Usage::TRANSFER_SRC,
        );
    
        let mut specular_image_state = SampledImageState::new(
            device_state.clone(),
            &adapter_state,
            specular_image,
            buffer::Usage::TRANSFER_SRC,
        );
        
        let mut vertices: Vec<Vertex> = vec![];
        /*
                for index in 0..chest_obj.position.len() {
                    let vertex = Vertex {
                        position: chest_obj.position[index],
                        normal: chest_obj.normal[index],
                        tangent: [0.0, 0.0, 0.0],
                        texture: chest_obj.texture[index],
                    };
                    vertices.push(vertex);
                }
                
                let mut indices: Vec<u32> = Vec::new();
                for object in chest_obj.objects {
                    for group in object.groups {
                        for polys in group.polys {
                            for face in polys.iter() {
                                indices.push(face.0 as u32);
                            }
                        }
                    }
                }*/
        for object in chest_obj.objects {
            for group in object.groups {
                for polys in group.polys {
                    let mut vertex_for_this_face: Vec<Vertex> = vec![];
                    for face in polys {
                        vertex_for_this_face.push(Vertex {
                            position: chest_obj.position[face.0],
                            normal: chest_obj.normal[face.2.unwrap()],
                            tangent: chest_obj.normal[face.2.unwrap()],
                            texture:
                            [
                                chest_obj.texture[face.1.unwrap()][0],
                                1.0 - chest_obj.texture[face.1.unwrap()][1]
                            ],
                        });
                    }
                    match vertex_for_this_face.len() {
                        3 => {
                            vertices.push(vertex_for_this_face[0]);
                            vertices.push(vertex_for_this_face[1]);
                            vertices.push(vertex_for_this_face[2]);
                        }
                        4 => {
                            vertices.push(vertex_for_this_face[0]);
                            vertices.push(vertex_for_this_face[1]);
                            vertices.push(vertex_for_this_face[2]);
                            vertices.push(vertex_for_this_face[0]);
                            vertices.push(vertex_for_this_face[2]);
                            vertices.push(vertex_for_this_face[3]);
                        }
                        5 => {
                            vertices.push(vertex_for_this_face[0]);
                            vertices.push(vertex_for_this_face[1]);
                            vertices.push(vertex_for_this_face[2]);
                            vertices.push(vertex_for_this_face[0]);
                            vertices.push(vertex_for_this_face[2]);
                            vertices.push(vertex_for_this_face[4]);
                            vertices.push(vertex_for_this_face[2]);
                            vertices.push(vertex_for_this_face[4]);
                            vertices.push(vertex_for_this_face[3]);
                        }
                        _ => panic!(),
                    };
                }
            }
        }
        
        let vertex_buffer = BufferState::new_from_items(
            device_state.clone(),
            &adapter_state,
            vertices,
            buffer::Usage::VERTEX,
        );
    
        /*        let indices_buffer = BufferState::new_from_items(
                    device_state.clone(),
                    &adapter_state,
                    indices,
                    buffer::Usage::INDEX,
                );
                */
        let camera = Camera::perspective(
            cgmath::Point3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            cgmath::Point3 {
                x: 2.0,
                y: -2.0,
                z: 2.0,
            },
        );
        
        let light = PointLight {
            position: cgmath::Point3 {
                x: 0.0,
                y: 0.0,
                z: 2.0,
            }
        };
    
        let vert_uniform_buffer = BufferState::new_from_items(
            device_state.clone(),
            &adapter_state,
            vec![
                VertUniformBlock {
                    projection_matrix: camera.projection.into(),
                    model_view_matrix: camera.view.into(),
                    normal_matrix: camera.normal.into(),
                    light_position: light.position.into(),
                }
            ],
            buffer::Usage::UNIFORM,
        );
        unsafe {
            let device = &device_state.borrow().device;
            device.write_descriptor_sets(
                vec![
                    DescriptorSetWrite {
                        set: vert_uniform_descriptor_state.descriptor_set.as_ref().unwrap(),
                        binding: 1,
                        array_offset: 0,
                        descriptors: &[
                            Descriptor::Buffer(
                                vert_uniform_buffer.buffer.as_ref().unwrap(),
                                None..None,
                            )
                        ],
                    }
                ]
            );
        }
        // order matters here!!
        normal_image_state.register_descriptor(
            &normal_descriptor_state
        );
        diffuse_image_state.register_descriptor(
            &diffuse_descriptor_state
        );
        specular_image_state.register_descriptor(
            &specular_descriptor_state
        );
    
        let object_pso = ObjectPso::new(
            device_state.clone(),
            render_pass_state.render_pass.as_ref().unwrap(),
            vec![
                &vert_uniform_descriptor_state,
                &normal_descriptor_state,
                &diffuse_descriptor_state,
                &specular_descriptor_state
            ],
            &mut gfs,
        );
    
        let mut staging_pool = {
            let device = &device_state.borrow().device;
            unsafe {
                device.create_command_pool_typed(
                    &device_state.borrow().queue_group,
                    CommandPoolCreateFlags::empty(),
                )
            }.unwrap()
        };
    
        let mut transferred_fence = {
            let device = &device_state.borrow_mut().device;
        
            device.create_fence(false)
        }.unwrap();
    
        unsafe {
            let normal_cp_cb = normal_image_state.transfer(&mut staging_pool);
            let diffuse_cp_cb = diffuse_image_state.transfer(&mut staging_pool);
            let specular_cp_cb = specular_image_state.transfer(&mut staging_pool);
        
            device_state.borrow_mut().queue_group.queues[0]
                .submit_nosemaphores(
                    &vec![
                        normal_cp_cb,
                        diffuse_cp_cb,
                        specular_cp_cb,
                    ],
                    Some(&mut transferred_fence),
                );
        
            let device = &device_state.borrow().device;
        
            device.wait_for_fence(&transferred_fence, !0).unwrap();
        
            device.destroy_command_pool(staging_pool.into_raw());
        }
    
        let rebuild_swapchain = false;
        RendererState {
            gfs,
            instance,
            surface,
            normal_descriptor_pool_state,
            diffuse_descriptor_pool_state,
            specular_descriptor_pool_state,
            adapter_state,
            vertex_buffer,
            vert_uniform_buffer,
            device_state,
            object_pso,
            render_pass_state,
            swapchain_state: Some(swapchain_state),
            viewport,
            rebuild_swapchain,
            vert_uniform_descriptor_state,
            normal_descriptor_state,
            diffuse_descriptor_state,
            specular_descriptor_state,
            vert_uniform_descriptor_pool_state,
            normal_image_state,
            diffuse_image_state,
            frame_buffer_state,
            //indices_buffer,
            specular_image_state,
        }
    }
    
    pub fn rebuild_swapchain(&mut self, render_size: Extent2D) {
        self.device_state.borrow().device.wait_idle().unwrap();
        self.rebuild_swapchain = false;
    
        self.swapchain_state.take().unwrap();
        self.swapchain_state = Some(SwapchainState::new(
            self.device_state.clone(),
            &self.adapter_state,
            &mut self.surface,
            render_size,
        ));
    
        self.render_pass_state = RenderPassState::new(
            self.device_state.clone(),
            self.swapchain_state.as_ref().unwrap(),
        );
        self.frame_buffer_state = FrameBufferState::new(
            self.device_state.clone(),
            &self.adapter_state,
            &self.render_pass_state,
            self.swapchain_state.as_mut().unwrap(),
        );
    
        self.viewport = RendererState::create_viewport(
            self.swapchain_state.as_ref().unwrap()
        );
    }
    
    pub fn try_rebuild_swapchain(&mut self, render_size: Extent2D) {
        if self.rebuild_swapchain {
            self.rebuild_swapchain(render_size);
        }
    }
    
    pub fn paint_frame(&mut self, camera: Camera, light: PointLight) {
        self.vert_uniform_buffer.update_buffer(
            vec![VertUniformBlock {
                projection_matrix: camera.projection.into(),
                model_view_matrix: camera.view.into(),
                normal_matrix: camera.normal.into(),
                light_position: light.position.into(),
            }]
        );
        
        let device_state = &mut self.device_state.borrow_mut();
        
        let semaphore_index = self.frame_buffer_state.current_index;
        
        // self.frame_buffer_state.increment_current_semaphores_index();
        
        let frame_index: SwapImageIndex = {
            let acquire_semaphore =
                &mut self.frame_buffer_state.acquire_semaphores.as_mut().unwrap()
                    [semaphore_index];
            match self.swapchain_state.as_mut().unwrap().swapchain.as_mut() {
                Some(swapchain) => {
                    match
                        unsafe {
                            swapchain.acquire_image(
                                !0,
                                FrameSync::Semaphore(
                                    acquire_semaphore))
                        } {
                        Ok(i) => i,
                        Err(_) => {
                            self.rebuild_swapchain = true;
                            return;
                        }
                    }
                }
                None => {
                    self.rebuild_swapchain = true;
                    return;
                }
            }
        };
        
        let swapchain = self.swapchain_state.as_ref().unwrap().swapchain.as_ref().unwrap();
        
        let (
            (
                frame_buffer_fence,
                frame_buffer,
                command_pool
            ),
            (
                image_acquired,
                image_present
            )
        ) = self.frame_buffer_state.get_frame_data(
            frame_index as usize,
            semaphore_index as usize,
        );
        
        unsafe {
            let device = &device_state.device;
            device
                .wait_for_fence(frame_buffer_fence, !0)
                .unwrap();
            device
                .reset_fence(frame_buffer_fence)
                .unwrap();
    
            command_pool.reset();
    
            let mut command_buffer =
                command_pool.acquire_command_buffer::<gfx_hal::command::OneShot>();
            command_buffer.begin();
    
            command_buffer.set_viewports(0, &[self.viewport.clone()]);
            command_buffer.set_scissors(0, &[self.viewport.rect.clone()]);
            command_buffer.bind_graphics_pipeline(self.object_pso.pipeline.as_ref().unwrap());
            command_buffer.bind_vertex_buffers(
                0,
                Some((self.vertex_buffer.buffer.as_ref().unwrap(), 0)),
            );
            /*            command_buffer.bind_index_buffer(
                            IndexBufferView {
                                buffer: self.indices_buffer.buffer.as_ref().unwrap(),
                                offset: 0,
                                index_type: IndexType::U32,
                            });*/
            command_buffer.bind_graphics_descriptor_sets(
                self.object_pso.pipeline_layout.as_ref().unwrap(),
                0,
                vec![
                    self.vert_uniform_descriptor_state.descriptor_set.as_ref().unwrap(),
                    self.normal_descriptor_state.descriptor_set.as_ref().unwrap(),
                    self.diffuse_descriptor_state.descriptor_set.as_ref().unwrap(),
                    self.specular_descriptor_state.descriptor_set.as_ref().unwrap(),
                ],
                &[],
            );
            {
                let mut encoder = command_buffer.begin_render_pass_inline(
                    self.render_pass_state.render_pass.as_ref().unwrap(),
                    frame_buffer,
                    self.viewport.rect.clone(),
                    &[ClearValue::Color(ClearColor::Float([0.0, 0.0, 0.0, 1.0])),
                        ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))],
                );
    
                encoder.draw(
                    0..((self.vertex_buffer.size.unwrap() / ::std::mem::size_of::<Vertex>() as
                        u64)
                        as u32),
                    0..1,
                );
            }
            command_buffer.finish();
    
            let submission = Submission {
                command_buffers: iter::once(&command_buffer),
                wait_semaphores: iter::once((&*image_acquired, PipelineStage::BOTTOM_OF_PIPE)),
                signal_semaphores: iter::once(&*image_present),
            };
            device_state.queue_group.queues[0]
                .submit(submission, Some(frame_buffer_fence));
            
            if let Err(_) = swapchain.present(
                &mut device_state.queue_group.queues[0],
                frame_index,
                Some(&*image_present),
            ) {
                self.rebuild_swapchain = true;
                return;
            }
        };
    }
    
    #[inline]
    pub fn create_viewport(swapchain_state: &SwapchainState) -> Viewport {
        Viewport {
            rect: Rect {
                x: 0,
                y: 0,
                w: swapchain_state.extent.width.clone() as i16,
                h: swapchain_state.extent.height.clone() as i16,
            },
            depth: 0.0..1.0,
        }
    }
}


enum RebuildError {
    Unspecified
}


fn try_rebuild_shader() -> Result<(), RebuildError> {
    for entry in std::fs::read_dir("res/shaders").unwrap() {
        let entry = entry.unwrap();
        
        if entry.file_type().unwrap().is_file() {
            let in_path = entry.path();
            
            // Support only vertex and fragment shaders currently
            let some_shader_type =
                in_path
                    .extension()
                    .and_then(|ext| match ext.to_string_lossy().as_ref() {
                        "vert" => Some(ShaderType::Vertex),
                        "frag" => Some(ShaderType::Fragment),
                        _ => None,
                    }
                    );
            
            if let Some(shader_type) = some_shader_type {
                use std::io::Read;
                
                let source = std::fs::read_to_string(&in_path).unwrap();
                let mut compiled_file = glsl_to_spirv::compile(&source, shader_type).unwrap();
                
                let mut compiled_bytes = Vec::new();
                compiled_file.read_to_end(&mut compiled_bytes).unwrap();
                
                let out_path = format!(
                    "res/shaders/gen/{}.spv",
                    in_path.file_name().unwrap().to_string_lossy()
                );
                
                std::fs::write(&out_path, &compiled_bytes).unwrap();
            }
        }
    };
    Ok(())
}
