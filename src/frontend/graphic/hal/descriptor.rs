use std::{
    cell::RefCell,
    rc::Rc,
};
use std::borrow::Borrow;

use super::{
    descriptor,
    device::DeviceState,
    prelude::*,
};

pub struct DescriptorPoolState {
    device_state: Rc<RefCell<DeviceState>>,
    descriptor_pool: Option<<B as TB>::DescriptorPool>,
}

impl DescriptorPoolState {
    pub fn new(
        device_state: Rc<RefCell<DeviceState>>,
        pool_size_descs: &[DescriptorRangeDesc],
    ) -> DescriptorPoolState {
        let descriptor_pool = unsafe {
            let device = &device_state.borrow_mut().device;
        
            let descriptor_pool =
                device.create_descriptor_pool(
                    1,
                    pool_size_descs,
                    pso::DescriptorPoolCreateFlags::empty(),
                ).ok();
            descriptor_pool
        };
    
        DescriptorPoolState {
            device_state,
            descriptor_pool,
        }
    }
}

impl Drop for DescriptorPoolState {
    fn drop(&mut self) {
        unsafe {
            let device = &self.device_state.borrow_mut().device;
            
            if let Some(dp) = self.descriptor_pool.take() {
                device.destroy_descriptor_pool(dp);
            }
        }
    }
}

pub struct DescriptorState {
    device_state: Rc<RefCell<DeviceState>>,
    pub descriptor_set_layout: Option<<B as TB>::DescriptorSetLayout>,
    pub descriptor_set: Option<<B as TB>::DescriptorSet>,
}

impl DescriptorState {
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
        
        DescriptorState {
            device_state,
            descriptor_set_layout,
            descriptor_set: None,
        }
    }
    pub fn allocate_descriptor_set(
        &mut self,
        descriptor_pool_state: &mut DescriptorPoolState,
    ) {
        if let Some(descriptor_set_layout) =
        self.descriptor_set_layout.as_ref()
        {
            let descriptor_pool =
                descriptor_pool_state.descriptor_pool.as_mut().unwrap();
            
            self.descriptor_set = unsafe {
                let descriptor_set =
                    descriptor_pool.allocate_set(
                        descriptor_set_layout
                    ).unwrap();
                Some(descriptor_set)
            }
        }
    }
}

impl Drop for DescriptorState {
    fn drop(&mut self) {
        unsafe {
            let device = &self.device_state.borrow_mut().device;
            
            if let Some(dsl) = self.descriptor_set_layout.take() {
                device.destroy_descriptor_set_layout(dsl);
            }
        }
    }
}