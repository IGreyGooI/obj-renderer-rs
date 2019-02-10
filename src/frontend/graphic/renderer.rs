use gfx_hal::Backend as TB;
use backend::Backend as B;

use gfx_hal::{
    adapter::{MemoryTypeId, Adapter, AdapterInfo},
    command::{BufferImageCopy, ClearColor, ClearDepthStencil, ClearValue},
    buffer,
    format::{Aspects, ChannelType, Format, Swizzle},
    image::{
        self as img, Access, Extent, Filter, Layout, Offset, SubresourceLayers, SubresourceRange,
        ViewCapabilities, ViewKind, WrapMode,
    },
    memory::{Barrier, Dependencies, Properties},
    pass::{
        Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, Subpass, SubpassDependency,
        SubpassDesc, SubpassRef,
    },
    pool::CommandPoolCreateFlags,
    pso::{
        AttributeDesc, BlendState, ColorBlendDesc, ColorMask, Comparison, DepthStencilDesc,
        DepthTest, Descriptor, DescriptorRangeDesc, DescriptorSetLayoutBinding, DescriptorSetWrite,
        DescriptorType, Element, EntryPoint, GraphicsPipelineDesc, GraphicsShaderSet,
        PipelineStage, Rasterizer, Rect, ShaderStageFlags, StencilTest, VertexBufferDesc, Viewport,
    },
    queue::Submission,
    window::Extent2D,
    Backbuffer, DescriptorPool, Device, FrameSync, Graphics, Instance, MemoryType, PhysicalDevice,
    Primitive, Surface, SwapImageIndex, Swapchain, SwapchainConfig, SurfaceCapabilities,
};

pub use winit::{
    WindowBuilder, Window, EventsLoop, dpi::LogicalSize, WindowEvent,
};
use super::constants::*;
use backend;
use std::{
    path,
    iter,
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::lib::resource::ReadFile;
use crate::lib::resource::gfs::GemFileSystem;
use super::window::WindowState;

pub struct Renderer {
    gfs: GemFileSystem<u8>,
}

pub struct DeviceState {
    pub device: <B as TB>::Device,
    pub queue_group: gfx_hal::QueueGroup<B, Graphics>,
    pub command_pool: Option<gfx_hal::CommandPool<B, Graphics>>,
}

impl DeviceState {
    fn new(
        adapter: &Adapter<B>,
        surface: &<B as TB>::Surface,
    ) -> Self {
        let (device, queue_group) = adapter
            .open_with::<_, Graphics>(
                1,
                |family| surface.supports_queue_family(family))
            .unwrap();
        let mut command_pool =
            unsafe {
                device.create_command_pool_typed(&queue_group, CommandPoolCreateFlags::empty())
            }.unwrap();
        DeviceState {
            device,
            queue_group,
            command_pool: Some(command_pool),
        }
    }
}

impl Drop for DeviceState {
    fn drop(&mut self) {
        let device = &self.device;
        unsafe {
            device.destroy_command_pool(self.command_pool.take().unwrap().into_raw());
        }
    }
}

pub struct RenderPassState {
    device_state: Rc<RefCell<DeviceState>>,
    render_pass: Option<<B as TB>::RenderPass>,
}

impl RenderPassState {
    fn new(swapchain_state: &SwapchainState, device_state: Rc<RefCell<DeviceState>>) -> Self {
        let render_pass =
            unsafe {
                let color_attachment = Attachment {
                    format: Some(swapchain_state.format),
                    samples: 1,
                    ops: AttachmentOps::new(AttachmentLoadOp::Clear, AttachmentStoreOp::Store),
                    stencil_ops: AttachmentOps::DONT_CARE,
                    layouts: Layout::Preinitialized..Layout::Present,
                };
                
                let subpass = SubpassDesc {
                    colors: &[(0, Layout::ColorAttachmentOptimal)],
                    depth_stencil: None,
                    inputs: &[],
                    resolves: &[],
                    preserves: &[],
                };
                
                let dependency = SubpassDependency {
                    passes: SubpassRef::External..SubpassRef::Pass(0),
                    stages: PipelineStage::COLOR_ATTACHMENT_OUTPUT..PipelineStage::COLOR_ATTACHMENT_OUTPUT,
                    accesses: Access::empty()
                        ..(Access::COLOR_ATTACHMENT_READ | Access::COLOR_ATTACHMENT_WRITE),
                };
                
                device_state.borrow().device.create_render_pass
                (&[color_attachment], &[subpass], &[dependency])
            }.unwrap();
        
        RenderPassState {
            render_pass: Some(render_pass),
            device_state,
        }
    }
}

impl Drop for RenderPassState {
    fn drop(&mut self) {
        let device = &self.device_state.borrow().device;
        unsafe {
            device.destroy_render_pass(self.render_pass.take().unwrap());
        }
    }
}

pub struct SwapchainState {
    pub build_required: bool,
    pub device_state: Rc<RefCell<DeviceState>>,
    pub extent: Extent,
    pub format: Format,
    pub swapchain: Option<<B as TB>::Swapchain>,
    pub frame_views: Option<Vec<<B as TB>::ImageView>>,
    pub frame_buffers: Option<Vec<<B as TB>::Framebuffer>>,
    pub frame_semaphore: Option<<B as TB>::Semaphore>,
    pub present_semaphore: Option<<B as TB>::Semaphore>,
}

impl SwapchainState {
    pub fn new(
        device_state: Rc<RefCell<DeviceState>>,
    ) -> SwapchainState {
        let frame_semaphore = {
            let device = &device_state.borrow().device;
            device.create_semaphore().unwrap()
        };
        let present_semaphore = {
            let device = &device_state.borrow().device;
            device.create_semaphore().unwrap()
        };
        SwapchainState {
            build_required: true,
            device_state,
            format: Format::Rgba8Srgb,
            extent: Extent::default(),
            swapchain: None,
            frame_views: None,
            frame_buffers: None,
            frame_semaphore: Some(frame_semaphore),
            present_semaphore: Some(present_semaphore),
        }
    }
    pub fn build(
        &mut self,
        swap_config: SwapchainConfig,
        surface: &mut <B as TB>::Surface,
        surface_color_format: Format,
        render_pass: <B as TB>::RenderPass,
    ) {
        let extent = swap_config.extent.to_extent();
        let device = &self.device_state.borrow().device;
        let (swapchain, backbuffer) = unsafe {
            //TODO: I cannot get a copy of swapchain which contains the ptr of actual swapchain
            // is there anything I can do here.
            device.create_swapchain(
                surface,
                swap_config,
                None,
            )
        }.unwrap();
        
        let (frame_views, frame_buffers) = unsafe {
            match backbuffer {
                Backbuffer::Images(images) => {
                    let color_range = SubresourceRange {
                        aspects: Aspects::COLOR,
                        levels: 0..1,
                        layers: 0..1,
                    };
                    
                    let image_views = images
                        .iter()
                        .map(
                            |image| {
                                device.create_image_view(
                                    image,
                                    ViewKind::D2,
                                    surface_color_format,
                                    Swizzle::NO,
                                    color_range.clone(),
                                ).unwrap()
                            }
                        )
                        .collect::<Vec<_>>();
                    let framebuffer = image_views
                        .iter()
                        .map(
                            |image_view| {
                                device
                                    .create_framebuffer(&render_pass,
                                                        vec![image_view],
                                                        self.extent,
                                    ).unwrap()
                            }
                        )
                        .collect();
                    (image_views, framebuffer)
                }
                Backbuffer::Framebuffer(framebuffer) => { (vec![], vec![framebuffer]) }
            }
        };
    }
}

impl Drop for SwapchainState {
    fn drop(&mut self) {
        let device = &self.device_state.borrow().device;
        unsafe {
            if let Some(frame_buffers) = self.frame_buffers.take() {
                for framebuffer in frame_buffers {
                    device.destroy_framebuffer(framebuffer);
                }
            }
            
            if let Some(frame_views) = self.frame_views.take() {
                for image_view in frame_views {
                    device.destroy_image_view(image_view);
                }
            }
            if let Some(swapchain) = self.swapchain.take() {
                device.destroy_swapchain(swapchain);
            }
            if let Some(frame_semaphore) = self.frame_semaphore.take() {
                device.destroy_semaphore(frame_semaphore)
            }
            if let Some(present_semaphore) = self.present_semaphore.take() {
                device.destroy_semaphore(present_semaphore)
            }
        }
    }
}

pub struct PipelineState {
    device_state: Rc<RefCell<DeviceState>>,
    pipeline: Option<<B as TB>::GraphicsPipeline>,
    pipeline_layout: Option<<B as TB>::PipelineLayout>,
    pipeline_cache: Option<<B as TB>::PipelineCache>,
}

impl PipelineState {
    pub fn new(
        device_state: Rc<RefCell<DeviceState>>,
        render_pass_state: &RenderPassState,
        gfs: &mut GemFileSystem<u8>,
    ) -> PipelineState {
        let pipeline_layout = unsafe {
            let device = &device_state.borrow().device;
            device.create_pipeline_layout(&[], &[])
        }.unwrap();
        
        let mut vertex_shader_module = {
            let spirv = gfs
                .load("shaders/gen/triangle.vert.spv".to_string())
                .expect("Cannot load shader");
            ShaderModuleState::new(device_state.clone(), spirv)
        };
        
        let mut fragment_shader_module = {
            let spirv = gfs
                .load("shaders/gen/triangle.frag.spv".to_string())
                .expect("Cannot load shader");
            ShaderModuleState::new(device_state.clone(), spirv)
        };
        let pipeline = unsafe {
            let device = &device_state.borrow().device;
            
            // call a struct's default() by directly referencing its trait of the function?
            // this works?
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
                main_pass: render_pass_state.render_pass.as_ref().unwrap(),
            };
            
            let mut pipeline_desc = GraphicsPipelineDesc::new(
                shader_set,
                Primitive::TriangleStrip,
                Rasterizer::FILL,
                &pipeline_layout,
                subpass,
            );
            pipeline_desc
                .blender
                .targets
                // what does Blending do?
                .push(ColorBlendDesc {
                    0: ColorMask::ALL,
                    1: BlendState::ALPHA,
                });
            
            device.create_graphics_pipeline(&pipeline_desc, None)
        }.unwrap();
        
        PipelineState {
            device_state,
            pipeline: Some(pipeline),
            pipeline_layout: Some(pipeline_layout),
            pipeline_cache: None,
        }
    }
}

impl Drop for PipelineState {
    fn drop(&mut self) {
        unsafe {
            let device = &self.device_state.borrow().device;
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

// a wrapper struct so that rust can drop memory at gpu when out of scope
// is option literally just a null ptr wrapper?
pub struct ShaderModuleState {
    //TODO: is Rc of device_state pointer necessary here?
    device_state: Rc<RefCell<DeviceState>>,
    module: Option<<B as TB>::ShaderModule>,
}

impl ShaderModuleState {
    pub fn new(
        device_state: Rc<RefCell<DeviceState>>,
        spirv: Box<[u8]>,
    ) -> ShaderModuleState {
        let module = unsafe {
            let device = &device_state.borrow().device;
            device.create_shader_module(spirv.as_ref())
        }.unwrap();
        ShaderModuleState {
            device_state,
            module: Some(module),
        }
    }
}

impl Drop for ShaderModuleState {
    fn drop(&mut self) {
        //TODO: my type checker in clion cannot show function of device when:
        // self.device_state.borrow().device.
        let device = &self.device_state.borrow().device;
        // it is just like: "if !ptr delete ptr"  idioms in cpp
        // since we are working on GPU memory
        unsafe {
            if let Some(module) = self.module.take() {
                device.destroy_shader_module(module)
            }
        }
    }
}

pub struct RendererState {
    // since the order of dropping matters due to need to call device.destory_*
    // the order of member matters here!
    gfs: GemFileSystem<u8>,
    pipeline_state: PipelineState,
    render_pass_state: RenderPassState,
    swapchain_state: SwapchainState,
    device_state: Rc<RefCell<DeviceState>>,
    instance: backend::Instance,
    adapter: Adapter<B>,
}

impl RendererState {
    pub fn new(
        window_state: &WindowState,
        render_size: Extent2D,
    ) -> RendererState {
        let mut events_loop = EventsLoop::new();
        
        let window = &window_state.window;
        
        let instance = backend::Instance::create(INSTANCE_NAME, 1);
        let surface = instance.create_surface(&window);
        
        let adapter = select_adapter(&mut instance.enumerate_adapters());
        
        let device_state =
            Rc::new(RefCell::new(DeviceState::new(&adapter, &surface)));
        
        let mut gfs = GemFileSystem::new(
            &concat!(env!("CARGO_MANIFEST_DIR"), "\\res")
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
        
        
        let pipeline_state = PipelineState::new(device_state.clone(), &render_pass_state, &mut gfs);
        
        RendererState {
            gfs,
            instance,
            adapter,
            device_state,
            pipeline_state,
            render_pass_state,
            swapchain_state,
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

pub trait Draw {}