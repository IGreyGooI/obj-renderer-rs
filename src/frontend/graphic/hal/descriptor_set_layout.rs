use super::{
    prelude::*,
    device::DeviceState,
};
use std::{
    rc::Rc,
    cell::RefCell,
};
use std::borrow::Borrow;

pub struct DescriptorSetLayoutState {
    device_state: Rc<RefCell<DeviceState>>,
    pub descriptor_set_layout: Option<<B as TB>::DescriptorSetLayout>,
}

impl DescriptorSetLayoutState {
    pub fn new(
        device_state: Rc<RefCell<DeviceState>>,
        sets: &[DescriptorSetLayoutBinding],
        immutable_samplers: &[<B as TB>::Sampler],
    ) -> Self {
        let descriptor_set_layout = unsafe {
            let device = &device_state.borrow_mut().device;
            
            device.create_descriptor_set_layout(
                sets,
                immutable_samplers,
            )
        }.ok();
        
        DescriptorSetLayoutState {
            device_state,
            descriptor_set_layout,
        }
    }
}

impl Drop for DescriptorSetLayoutState {
    fn drop(&mut self) {
        unsafe {
            let device = &self.device_state.borrow_mut().device;
            
            if let Some(dsl) = self.descriptor_set_layout.take() {
                device.destroy_descriptor_set_layout(dsl);
            }
        }
    }
}