use super::{
    prelude::*,
    device::DeviceState
};
use std::{
    rc::Rc,
    cell::RefCell,
    marker::PhantomData
};

pub struct UniformBuffer<U: Sized + Copy> {
    buffer: Option<<B as TB>::Buffer>,
    memory: Option<<B as TB>::Memory>,
    device_state: Rc<RefCell<DeviceState>>,
    _phantom_data: PhantomData<U>,
}

impl<U: Sized + Copy> UniformBuffer<U> {
    pub fn new(device_state: Rc<RefCell<DeviceState>>,
               adapter: Adapter<B>,
               items: Vec<U>,
               stage: PipelineStage,
    ) -> UniformBuffer<U> {
        let physical_device = adapter.physical_device;
        
        let memory_types = physical_device.memory_properties().memory_types;
        
        let (buffer, memory) = unsafe {
            let device = &device_state.borrow_mut().device;
            let (buffer, memory) = super::util::buffer::create_buffer(
                device,
                memory_types,
                Properties::CPU_VISIBLE,
                buffer::Usage::UNIFORM,
                items);
            (Some(buffer), Some(memory))
        };
        
        UniformBuffer {
            device_state,
            buffer,
            memory,
            _phantom_data: PhantomData,
        }
    }
}

impl<U: Sized + Copy> Drop for UniformBuffer<U> {
    fn drop(&mut self) {
        unsafe {
            let device = &self.device_state.borrow().device;
            if let Some(buffer) = self.buffer.take() {
                device.destroy_buffer(buffer);
            }
            if let Some(memory) = self.memory.take() {
                device.free_memory(memory)
            }
        }
    }
}

pub struct VertexBuffer<V: Sized + Copy> {
    buffer: Option<<B as TB>::Buffer>,
    memory: Option<<B as TB>::Memory>,
    device_state: Rc<RefCell<DeviceState>>,
    _phantom_data: PhantomData<V>,
}

impl<V: Sized + Copy> VertexBuffer<V> {
    pub fn new(device_state: Rc<RefCell<DeviceState>>,
               adapter: &Adapter<B>,
               vertices: Vec<V>,
    ) -> VertexBuffer<V> {
        let physical_device = &adapter.physical_device;
        
        let memory_types = physical_device.memory_properties().memory_types;
        
        let (buffer, memory) = unsafe {
            let device = &device_state.borrow_mut().device;
            let (buffer, memory) = super::util::buffer::create_buffer(
                device,
                memory_types,
                Properties::CPU_VISIBLE,
                buffer::Usage::VERTEX,
                vertices);
            (Some(buffer), Some(memory))
        };
        
        VertexBuffer {
            device_state,
            buffer,
            memory,
            _phantom_data: PhantomData,
        }
    }
}

impl<V: Sized + Copy> Drop for VertexBuffer<V> {
    fn drop(&mut self) {
        unsafe {
            let device = &self.device_state.borrow().device;
            if let Some(buffer) = self.buffer.take() {
                device.destroy_buffer(buffer);
            }
            if let Some(memory) = self.memory.take() {
                device.free_memory(memory)
            }
        }
    }
}
