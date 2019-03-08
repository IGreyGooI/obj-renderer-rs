use std::{
    cell::RefCell,
    rc::Rc,
};

use crate::frontend::graphic::constants::COLOR_RANGE;
use crate::frontend::graphic::hal::render_pass::RenderPassState;

use super::{
    device::DeviceState,
    prelude::*,
};
use super::adapter::AdapterState;

pub struct SwapchainState {
    pub device_state: Rc<RefCell<DeviceState>>,
    pub swapchain: Option<<B as TB>::Swapchain>,
    pub backbuffer: Option<Backbuffer<B>>,
    pub extent: Extent,
    pub color_format: Format,
    pub depth_format: Format,
}

impl SwapchainState {
    pub fn new(
        device_state: Rc<RefCell<DeviceState>>,
        adapter: &AdapterState,
        surface: &mut <B as TB>::Surface,
        extent: Extent2D,
    ) -> SwapchainState {
        let (caps, formats, _present_modes) =
            surface.compatibility(&adapter.adapter.physical_device);
        println!("[INFO][Formats]{:?}", formats);
    
        let color_format = select_surface_color_format(formats);
        println!("[INFO][Chosen Surface Format] {:?}", color_format);
    
        let depth_format = Format::D32SfloatS8Uint;
        println!("[INFO][Chosen Depth Format] {:?}", depth_format);
        
        let swap_config =
            SwapchainConfig::from_caps(&caps, color_format, extent);
        
        let extent = swap_config.extent.to_extent();
    
        let (swapchain, backbuffer) =
            unsafe {
                let device = &device_state.borrow().device;
    
                device
                    .create_swapchain(
                        surface,
                        swap_config,
                        None,
                    )
            }.expect("Can't create swapchain");
        
        SwapchainState {
            swapchain: Some(swapchain),
            backbuffer: Some(backbuffer),
            device_state,
            extent,
            color_format,
            depth_format,
        }
    }
}

impl Drop for SwapchainState {
    fn drop(&mut self) {
        let device = &self.device_state.borrow().device;
        unsafe {
            if let Some(swapchain) = self.swapchain.take() {
                device.destroy_swapchain(swapchain);
            }
        }
    }
}

pub struct FrameBufferState {
    pub command_pools: Option<Vec<CommandPool<B, Graphics>>>,
    pub frame_buffers: Option<Vec<<B as TB>::Framebuffer>>,
    pub frame_buffer_fences: Option<Vec<<B as TB>::Fence>>,
    pub frame_images: Option<Vec<<B as TB>::Image>>,
    pub frame_image_views: Option<Vec<<B as TB>::ImageView>>,
    pub acquire_semaphores: Option<Vec<<B as TB>::Semaphore>>,
    pub present_semaphores: Option<Vec<<B as TB>::Semaphore>>,
    pub depth_image: Option<<B as TB>::Image>,
    pub depth_image_memory: Option<<B as TB>::Memory>,
    pub depth_image_view: Option<<B as TB>::ImageView>,
    pub current_index: usize,
    pub last_index: usize,
    pub device_state: Rc<RefCell<DeviceState>>,
}

impl FrameBufferState {
    pub fn new(
        device_state: Rc<RefCell<DeviceState>>,
        adapter_state: &AdapterState,
        render_pass: &RenderPassState,
        swapchain: &mut SwapchainState,
    ) -> Self {
        let extent = Extent {
            width: swapchain.extent.width as _,
            height: swapchain.extent.height as _,
            depth: 1,
        };
    
    
        let (depth_image, depth_image_memory, depth_image_view) = unsafe {
            let device = &device_state.borrow().device;
        
            let kind =
                image::Kind::D2(extent.width as image::Size, extent.height as
                    image::Size, 1, 1);
        
            let mut depth_image = device
                .create_image(
                    kind,
                    1,
                    swapchain.depth_format,
                    image::Tiling::Optimal,
                    image::Usage::DEPTH_STENCIL_ATTACHMENT,
                    ViewCapabilities::empty(),
                )
                .expect("Failed to create unbound depth image");
        
            let image_req = device.get_image_requirements(&depth_image);
        
            let device_type = adapter_state.memory_types
                                           .iter()
                                           .enumerate()
                                           .position(|(id, memory_type)| {
                                               image_req.type_mask & (1 << id) != 0
                                                   && memory_type.properties.contains(Properties::DEVICE_LOCAL)
                                           })
                                           .unwrap()
                                           .into();
        
            let depth_image_memory = device
                .allocate_memory(device_type, image_req.size)
                .expect("Failed to allocate depth image");
        
            device
                .bind_image_memory(&depth_image_memory, 0, &mut depth_image)
                .expect("Failed to bind depth image");
        
            let depth_image_view = device
                .create_image_view(
                    &depth_image,
                    image::ViewKind::D2,
                    swapchain.depth_format,
                    Swizzle::NO,
                    image::SubresourceRange {
                        aspects: Aspects::DEPTH | Aspects::STENCIL,
                        levels: 0..1,
                        layers: 0..1,
                    },
                )
                .expect("Failed to create image view");
        
            (depth_image, depth_image_memory, depth_image_view)
        };
        
        let (frame_images, frame_image_views, frame_buffers) = {
            let device = &device_state.borrow().device;
    
    
            match swapchain.backbuffer.take().unwrap() {
                Backbuffer::Images(frame_images) => {
                    let frame_image_views =
                        frame_images
                            .iter()
                            .map(|image| {
                                let frame_image_view = unsafe {
                                    device.create_image_view(
                                        &image,
                                        ViewKind::D2,
                                        swapchain.color_format,
                                        Swizzle::NO,
                                        COLOR_RANGE.clone(),
                                    )
                                }.unwrap();
                                frame_image_view
                            }).collect::<Vec<_>>();
    
                    let frame_buffers = frame_image_views.iter().map(
                        |image_view| {
                            unsafe {
                                device.create_framebuffer(
                                    render_pass.render_pass.as_ref().unwrap(),
                                    vec![image_view, &depth_image_view],
                                    extent,
                                )
                            }.unwrap()
                        }
                    ).collect::<Vec<_>>();
                    ;
                    (frame_images, frame_image_views, frame_buffers)
                }
                Backbuffer::Framebuffer(fbo) =>
                    (Vec::new(), Vec::new(), vec![fbo])
            }
        };
        
        let iter_count = if frame_images.len() != 0 {
            frame_images.len()
        } else {
            1 // GL can have zero
        };
        
        let mut fences: Vec<<B as TB>::Fence> = vec![];
        let mut command_pools: Vec<CommandPool<B, Graphics>> = vec![];
        let mut acquire_semaphores: Vec<<B as TB>::Semaphore> = vec![];
        let mut present_semaphores: Vec<<B as TB>::Semaphore> = vec![];
        {
            let device = &device_state.borrow().device;
            for _ in 0..iter_count {
                fences.push(device.create_fence(true).unwrap());
                command_pools.push(
                    unsafe {
                        device
                            .create_command_pool_typed(
                                &device_state.borrow().queue_group,
                                CommandPoolCreateFlags::empty(),
                            )
                    }.expect("Can't create command pool"),
                );
                
                acquire_semaphores.push(device.create_semaphore().unwrap());
                present_semaphores.push(device.create_semaphore().unwrap());
            }
        }
        
        
        FrameBufferState {
            frame_images: Some(frame_images),
            frame_image_views: Some(frame_image_views),
            frame_buffers: Some(frame_buffers),
            frame_buffer_fences: Some(fences),
            command_pools: Some(command_pools),
            present_semaphores: Some(present_semaphores),
            depth_image: Some(depth_image),
            depth_image_memory: Some(depth_image_memory),
            acquire_semaphores: Some(acquire_semaphores),
            device_state,
            current_index: 0,
            last_index: 0,
            depth_image_view: Some(depth_image_view)
        }
    }
    
    pub fn increment_current_semaphores_index(&mut self) {
        let num_of_acquire_semaphores = self.acquire_semaphores.as_ref().unwrap().len();
        self.current_index += 1;
        if self.current_index >= num_of_acquire_semaphores {
            self.current_index = 0
        }
    }
    
    pub fn get_frame_data(
        &mut self,
        frame_index: usize,
        semaphore_index: usize,
    ) -> (
        (
            &mut <B as TB>::Fence,
            &mut <B as TB>::Framebuffer,
            &mut CommandPool<B, Graphics>
        ),
        (
            &mut <B as TB>::Semaphore,
            &mut <B as TB>::Semaphore
        ),
    ) {
        (
            (
                &mut self.frame_buffer_fences.as_mut().unwrap()[frame_index],
                &mut self.frame_buffers.as_mut().unwrap()[frame_index],
                &mut self.command_pools.as_mut().unwrap()[frame_index],
            ),
            (
                &mut self.acquire_semaphores.as_mut().unwrap()[semaphore_index],
                &mut self.present_semaphores.as_mut().unwrap()[semaphore_index],
            )
        )
    }
}

impl Drop for FrameBufferState {
    fn drop(&mut self) {
        let device = &self.device_state.borrow().device;
    
        unsafe {
            if let Some(frame_buffer_fences) =
            self.frame_buffer_fences.take() {
                for fence in frame_buffer_fences {
                    device.destroy_fence(fence);
                }
            }
            if let Some(command_pools) =
            self.command_pools.take() {
                for command_pool in command_pools {
                    device.destroy_command_pool(command_pool.into_raw());
                }
            }
            if let Some(acquire_semaphores) =
            self.acquire_semaphores.take() {
                for acquire_semaphore in acquire_semaphores {
                    device.destroy_semaphore(acquire_semaphore);
                }
            }
            if let Some(present_semaphores) =
            self.present_semaphores.take() {
                for present_semaphore in present_semaphores {
                    device.destroy_semaphore(present_semaphore);
                }
            }
            if let Some(frame_buffers) =
            self.frame_buffers.take() {
                for framebuffer in frame_buffers {
                    device.destroy_framebuffer(framebuffer);
                }
            }
            if let Some(frame_image_views) =
            self.frame_image_views.take() {
                for frame_image in frame_image_views {
                    device.destroy_image_view(frame_image);
                }
            }
        }
    }
}


pub fn select_surface_color_format(formats: Option<Vec<Format>>) -> Format {
    formats.map_or(Format::Rgba8Srgb, |formats| {
        formats
            .iter()
            .find(|format| format.base_format().1 == ChannelType::Srgb)
            .map(|format| *format)
            .unwrap_or(formats[0])
    })
}