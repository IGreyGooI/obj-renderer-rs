use super::{
    prelude::*,
    device::DeviceState,
    descriptor_set_layout::DescriptorSetLayoutState,
};
use std::{
    rc::Rc,
    cell::RefCell,
};

pub struct DescriptorState {
    device_state: Rc<RefCell<DeviceState>>,
    descriptor_pool: Option<<B as TB>::DescriptorPool>,
    descriptor_set: Option<<B as TB>::DescriptorSet>,
}

impl DescriptorState {
    pub fn new(
        device_state: Rc<RefCell<DeviceState>>,
        descriptor_set_layout_state: DescriptorSetLayoutState,
    ) -> DescriptorState {
        let (descriptor_pool, descriptor_set) = unsafe {
            let device = &device_state.borrow_mut().device;
            
            if let Some(mut descriptor_pool) =
            device.create_descriptor_pool(
                1,
                &[
                    DescriptorRangeDesc {
                        ty: DescriptorType::UniformBuffer,
                        count: 1,
                    }
                ],
            ).ok()
            {
                let descriptor_set =
                    match descriptor_set_layout_state.descriptor_set_layout.as_ref(){
                        Some(descriptor_set_layout) => {
                            let descriptor_set =
                                descriptor_pool.allocate_set(descriptor_set_layout)
                                               .ok();
                            descriptor_set
                        }
                        None => None
                    };
                
                (Some(descriptor_pool), descriptor_set)
            } else {
                (None, None)
            }
        };
        
        DescriptorState {
            device_state,
            descriptor_pool,
            descriptor_set,
        }
    }
}