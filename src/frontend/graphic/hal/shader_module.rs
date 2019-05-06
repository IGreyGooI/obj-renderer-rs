use std::{
    cell::RefCell,
    rc::Rc,
};

use super::{
    device::DeviceState,
    prelude::*,
};

pub struct ShaderModuleState {
    //TODO: is Rc of device_state pointer necessary here?
    device_state: Rc<RefCell<DeviceState>>,
    pub module: Option<<B as TB>::ShaderModule>,
}

impl ShaderModuleState {
    pub fn new(
        device_state: Rc<RefCell<DeviceState>>,
        spirv: &Box<[u8]>,
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
        let device = &self.device_state.borrow().device;
        
        // it is just like: "if !ptr delete ptr" idioms in cpp
        // since we are working on GPU memory
        unsafe {
            if let Some(module) = self.module.take() {
                device.destroy_shader_module(module)
            }
        }
    }
}
