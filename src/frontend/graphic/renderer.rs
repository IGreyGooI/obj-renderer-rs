use crate::{
    lib::resource::gfs::GemFileSystem,
    frontend::{
        graphic::{
            data_type::*,
            hal::{
                render_pass::RenderPassState,
                pipeline::ObjectPso,
                swapchain::SwapchainState,
                device::DeviceState,
                prelude::*,
                descriptor_set_layout::DescriptorSetLayoutState,
                buffer::{UniformBuffer, VertexBuffer},
            }
        }
    },
};
use std::{
    cell::RefCell,
    rc::Rc,
    iter,
};
use backend;
use super::window::WindowState;
use super::constants::*;
use crate::lib::resource::ReadFile;

pub struct RendererState {
    // Rendering Global Variables:
    pub viewport: Viewport,
    pub rebuild_swapchain: bool,
    /// adapter in use for this Renderer
    pub adapter: Adapter<B>,
    
    // The Following are the ones implemented Drop
    // since they own part of memory on device
    // and dropping them need to call device.destory_*
    // Thus the order of dropping matters,
    // which is reflected as the order of members here!
    vertex_buffer: VertexBuffer<Vertex>,
    vert_uniform_buffer: UniformBuffer<VertUniformBlock>,
    frag_uniform_buffer: UniformBuffer<FragUniformBlock>,
    gfs: GemFileSystem < u8 >,
    object_pso: ObjectPso,
    render_pass_state: RenderPassState,
    swapchain_state: SwapchainState,
    device_state: Rc<RefCell<DeviceState>>,
    // Instance should be drop last one
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
        let surface = instance.create_surface(&window);
        let adapter = select_adapter(&mut instance.enumerate_adapters());
        let device_state =
            Rc::new(RefCell::new(DeviceState::new(&adapter, &surface)));
        
        let mut gfs = GemFileSystem::new(
            &concat!(env ! ("CARGO_MANIFEST_DIR"), "\\res")
        );
        let physical_device = &adapter.physical_device;
        
        let (caps, formats, _) = surface.compatibility(physical_device);
        
        let surface_color_format = select_surface_color_format(formats);
        
        let swap_config = SwapchainConfig::from_caps(
            &caps,
            surface_color_format,
            render_size,
        );
        
        let swapchain_state = SwapchainState::new(device_state.clone());
        
        let render_pass_state = RenderPassState::new(&swapchain_state, device_state.clone());
        
        let descriptor_set_layout_state = DescriptorSetLayoutState::new(
            device_state.clone(),
            &[
                DescriptorSetLayoutBinding {
                    binding: 0,
                    ty: DescriptorType::UniformBuffer,
                    count: 0,
                    stage_flags: ShaderStageFlags::VERTEX,
                    immutable_samplers: false,
                },
                DescriptorSetLayoutBinding {
                    binding: 0,
                    ty: DescriptorType::UniformBuffer,
                    count: 0,
                    stage_flags: ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                },
                DescriptorSetLayoutBinding {
                    binding: 1,
                    ty: DescriptorType::SampledImage,
                    count: 0,
                    stage_flags: ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                },
                DescriptorSetLayoutBinding {
                    binding: 2,
                    ty: DescriptorType::SampledImage,
                    count: 0,
                    stage_flags: ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                },
                DescriptorSetLayoutBinding {
                    binding: 3,
                    ty: DescriptorType::SampledImage,
                    count: 0,
                    stage_flags: ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                },
            ],
            &[],
        );
        
        
        let object_pso = ObjectPso::new(
            device_state.clone(),
            render_pass_state.render_pass.as_ref().unwrap(),
            &descriptor_set_layout_state,
            &mut gfs,
        );
        
        let viewport = Viewport {
            rect: Rect {
                x: 0,
                y: 0,
                w: render_size.width as i16,
                h: render_size.height as i16,
            },
            depth: 0.0..1.0,
        };
        
        let model_file = gfs.load("models/Chest.obj".to_string()).unwrap();
        let object = obj::Obj::load_buf(model_file.as_ref()).unwrap();
        
        let vertex_buffer = VertexBuffer::new(
            device_state,
            &adapter,
            7a2ocmqios
        )
        
        
        let rebuild_swapchain = true;
        
        RendererState {
            gfs,
            instance,
            surface,
            adapter,
            device_state,
            object_pso,
            render_pass_state,
            swapchain_state,
            viewport,
            rebuild_swapchain,
        }
    }
    
    pub fn rebuild_swapchain(&mut self, render_size: Extent2D) {
        self.rebuild_swapchain = false;
        let surface = &mut self.surface;
        let (caps, formats, _) = surface.compatibility(&self.adapter.physical_device);
        
        let surface_color_format = select_surface_color_format(formats);
        
        let swap_config = SwapchainConfig::from_caps(
            &caps,
            surface_color_format,
            render_size,
        );
        
        let render_pass = self.render_pass_state.render_pass.as_ref().unwrap();
        let swapchain_state = &mut self.swapchain_state;
        swapchain_state.build(
            swap_config,
            surface,
            surface_color_format,
            render_pass,
        )
    }
    
    pub fn try_rebuild_swapchain(&mut self, render_size: Extent2D) {
        if self.rebuild_swapchain {
            self.rebuild_swapchain(render_size);
        }
    }
    
    pub fn paint_frame(&mut self) {
        self.viewport = self.create_viewport();
        let mut device_state = self.device_state.borrow_mut();
        let command_pool = device_state.command_pool.as_mut().unwrap();
        
        unsafe {
            command_pool.reset();
        }
        
        let frame_index: SwapImageIndex = {
            match self.swapchain_state.swapchain.as_mut() {
                Some(swapchain) => {
                    match
                        unsafe {
                            swapchain.acquire_image(
                                !0,
                                FrameSync::Semaphore(
                                    self.swapchain_state
                                        .frame_semaphore.as_ref().unwrap()))
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
        
        let swapchain = self.swapchain_state.swapchain.as_ref().unwrap();
        
        let mut command_buffer = unsafe {
            command_pool.acquire_command_buffer::<gfx_hal::command::OneShot>()
        };
        unsafe {
            command_buffer.begin();
            command_buffer.set_viewports(0, &[self.viewport.clone()]);
            command_buffer.set_scissors(0, &[self.viewport.rect.clone()]);
            command_buffer.bind_graphics_pipeline(self.object_pso.pipeline.as_ref().unwrap());
            command_buffer.bind_vertex_buffer(0);
            {
                let mut encoder = command_buffer.begin_render_pass_inline(
                    self.render_pass_state.render_pass.as_ref().unwrap(),
                    &self.swapchain_state.frame_buffers.as_ref().unwrap()[frame_index as usize],
                    self.viewport.rect.clone(),
                    &[ClearValue::Color(ClearColor::Float([0.0, 0.0, 0.0, 1.0]))],
                );
                
                encoder.draw(0..12, 0..1);
            }
            command_buffer.finish();
        }
        unsafe {
            let frame_semaphore = self.swapchain_state.frame_semaphore.as_ref().unwrap();
            let present_semaphore = self.swapchain_state.present_semaphore.as_ref().unwrap();
            let submission = Submission {
                command_buffers: iter::once(&command_buffer),
                wait_semaphores: iter::once((frame_semaphore, PipelineStage::BOTTOM_OF_PIPE)),
                signal_semaphores: iter::once(present_semaphore),
            };
            
            let command_queue = &mut device_state.queue_group.queues[0];
            command_queue.submit(submission, None);
            
            if let Err(_) = swapchain.present(
                command_queue,
                frame_index,
                Some(self.swapchain_state.present_semaphore.as_ref().unwrap()),
            ) {
                self.rebuild_swapchain = true;
                return;
            }
        };
    }
    
    pub fn acquire_command_buffer(&mut self) -> CommandBuffer<B, Graphics> {
        let mut device_state = self.device_state.borrow_mut();
        let mut command_pool = device_state.command_pool.as_mut().unwrap();
        command_pool.acquire_command_buffer::<gfx_hal::command::OneShot>()
    }
    
    #[inline]
    pub fn create_viewport(&self) -> Viewport {
        Viewport {
            rect: Rect {
                x: 0,
                y: 0,
                w: self.swapchain_state.extent.width.clone() as i16,
                h: self.swapchain_state.extent.height.clone() as i16,
            },
            depth: 0.0..1.0,
        }
    }
}

pub fn select_adapter(adapters: &mut Vec<Adapter<B>>) -> Adapter<B> {
    adapters.remove(0)
}

pub fn select_surface_color_format(formats: Option<Vec<Format>>) -> Format {
    match formats {
        Some(choices) => choices
            .into_iter()
            .find(|format| format.base_format().1 == ChannelType::Srgb)
            .unwrap(),
        None => Format::Rgba8Srgb,
    }
}