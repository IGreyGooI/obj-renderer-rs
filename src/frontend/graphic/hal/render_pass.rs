use std::{
    rc::Rc,
    cell::RefCell
};
use super::{
    device::DeviceState,
    prelude::*,
    swapchain::SwapchainState
};

pub struct RenderPassState {
    device_state: Rc<RefCell<DeviceState>>,
    pub render_pass: Option<<B as TB>::RenderPass>,
}

impl RenderPassState {
    pub fn new(swapchain_state: &SwapchainState, device_state: Rc<RefCell<DeviceState>>) -> Self {
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
                    stages: PipelineStage::COLOR_ATTACHMENT_OUTPUT
                        ..PipelineStage::COLOR_ATTACHMENT_OUTPUT,
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
