use super::{
    prelude::*,
    device::DeviceState
};
use std::{
    rc::Rc,
    cell::RefCell
};

pub struct SwapchainState {
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
        render_pass: &<B as TB>::RenderPass,
    ) {
        let device = &self.device_state.borrow().device;
        
        let extent = swap_config.extent.to_extent();
        
        let (swapchain, backbuffer) = unsafe {
            if let Some(swapchain) = self.swapchain.take() {
                device.create_swapchain(
                    surface,
                    swap_config,
                    Some(swapchain),
                )
            } else {
                device.create_swapchain(
                    surface,
                    swap_config,
                    None,
                )
            }
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
                                    .create_framebuffer(render_pass,
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
        
        self.swapchain = Some(swapchain);
        self.frame_views = Some(frame_views);
        self.frame_buffers = Some(frame_buffers);
        self.extent = extent;
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
