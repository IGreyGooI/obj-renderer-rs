use std::{
    cell::RefCell,
    marker::PhantomData,
    rc::Rc,
};

use super::{
    adapter::AdapterState,
    device::DeviceState,
    prelude::*,
};

pub struct BufferState<I> {
    pub buffer: Option<<B as TB>::Buffer>,
    pub memory: Option<<B as TB>::Memory>,
    pub size: Option<u64>,
    pub device_state: Rc<RefCell<DeviceState>>,
    pub _phantom_data: PhantomData<I>,
}

impl<I: Sized + Copy> BufferState<I> {
    pub fn new_from_items(
        device_state: Rc<RefCell<DeviceState>>,
        adapter_state: &AdapterState,
        items: Vec<I>,
        usage: buffer::Usage,
    ) -> BufferState<I> {
        let (buffer, memory, size) = unsafe {
            let device = &device_state.borrow_mut().device;
            let (buffer, memory, size) = create_buffer(
                device,
                adapter_state,
                Properties::CPU_VISIBLE,
                usage,
                items);
            (Some(buffer), Some(memory), Some(size))
        };
        
        BufferState {
            buffer,
            memory,
            size,
            device_state,
            _phantom_data: PhantomData,
        }
    }
    pub fn new_texture_buffer(
        device_state: Rc<RefCell<DeviceState>>,
        adapter_state: &AdapterState,
        width: u32,
        height: u32,
        bytes_pixels: u32,
        image: Vec<I>,
        usage: buffer::Usage,
    ) -> (BufferState<I>, u32)
    {
        let (buffer, memory, size, pitch_size) = unsafe {
            let device = &device_state.borrow_mut().device;
            let (buffer, mut memory, size, pitch_size) =
                create_empty_buffer_with_chunk_size_align_to_size::<u8>(
                    device,
                    &adapter_state.memory_types,
                    Properties::CPU_VISIBLE,
                    usage,
                    height,
                    width * bytes_pixels,
                    adapter_state.limits.min_buffer_copy_pitch_alignment as u32);
            fill_buffer_with_alignment(
                device,
                &mut memory,
                image,
                height,
                width * bytes_pixels,
                pitch_size,
                size,
            );
            (Some(buffer), Some(memory), Some(size), pitch_size)
        };
        (BufferState {
            buffer,
            memory,
            size,
            device_state,
            _phantom_data: PhantomData,
        }, pitch_size
        )
    }
    pub fn update_buffer(&mut self, items: Vec<I>) {
        unsafe {
            fill_buffer(
                &self.device_state.borrow().device,
                self.memory.as_mut().unwrap(),
                items,
                self.size.unwrap(),
            );
        }
    }
}


unsafe fn create_buffer<I: Copy>(
    device: &<B as TB>::Device,
    adapter_state: &AdapterState,
    properties: Properties,
    usage: buffer::Usage,
    items: Vec<I>,
) -> (<B as TB>::Buffer, <B as TB>::Memory, u64) {
    let memory_types = &adapter_state.memory_types;
    let (buffer, mut memory, size) = create_empty_buffer::<I>(
        device,
        memory_types,
        properties,
        usage,
        items.len() as u64,
    );
    
    fill_buffer(
        device,
        &mut memory,
        items,
        size,
    );
    (buffer, memory, size)
}

unsafe fn create_empty_buffer<I>(
    device: &<B as TB>::Device,
    memory_types: &Vec<MemoryType>,
    properties: Properties,
    usage: buffer::Usage,
    item_count: u64,
) -> (<B as TB>::Buffer, <B as TB>::Memory, u64) {
    let stride = ::std::mem::size_of::<I>() as u64;
    let upload_size = item_count * stride;
    let mut buffer = device.create_buffer(upload_size, usage).unwrap();
    let req = device.get_buffer_requirements(&buffer);
    let upload_type = memory_types
        .iter()
        .enumerate()
        .position(|(id, ty)| req.type_mask & (1 << id) != 0 && ty.properties.contains(properties))
        .unwrap()
        .into();
    let memory = device.allocate_memory(upload_type, req.size).unwrap();
    // TODO: error-check?
    device.bind_buffer_memory(&memory, 0, &mut buffer)
          .expect("failed to bind buffer memory");
    (buffer, memory, req.size)
}

/// Pushes data into a buffer.
unsafe fn fill_buffer<I: Copy>(
    device: &<B as TB>::Device,
    buffer_memory: &mut <B as TB>::Memory,
    items: Vec<I>,
    size: u64,
) {
    // NOTE: MESH -> items
    // NOTE: Recalc buffer_len
    let buffer_size = size;
    
    let mut dest = device
        .acquire_mapping_writer::<I>(&buffer_memory, 0..buffer_size)
        .unwrap();
    
    dest[0..items.len()].copy_from_slice(items.as_slice());
    device.release_mapping_writer(dest).expect("failed to release mapping writer");
}


unsafe fn create_empty_buffer_with_size(
    device: &<B as TB>::Device,
    memory_types: &Vec<MemoryType>,
    properties: Properties,
    usage: buffer::Usage,
    buffer_size: u64) -> (<B as TB>::Buffer, <B as TB>::Memory, u64)
{
    let mut buffer = device.create_buffer(buffer_size, usage).unwrap();
    let req = device.get_buffer_requirements(&buffer);
    let upload_type = memory_types
        .iter()
        .enumerate()
        .position(|(id, ty)| req.type_mask & (1 << id) != 0 && ty.properties.contains(properties))
        .unwrap()
        .into();
    let memory = device.allocate_memory(upload_type, req.size).unwrap();
    // TODO: error-check?
    device.bind_buffer_memory(&memory, 0, &mut buffer)
          .expect("failed to bind buffer memory");
    (buffer, memory, req.size)
}

unsafe fn create_empty_buffer_with_chunk_size_align_to_size<I>(
    device: &<B as TB>::Device,
    memory_types: &Vec<MemoryType>,
    properties: Properties,
    usage: buffer::Usage,
    num_of_chunk: u32,
    items_count_each_chunk: u32,
    alignment: u32,
) -> (<B as TB>::Buffer, <B as TB>::Memory, u64, u32) {
    let alignment_mask = alignment - 1;
    let stride = ::std::mem::size_of::<I>();
    
    let chunk_size = (items_count_each_chunk * stride as u32 + alignment_mask) & !alignment_mask;
    let upload_size = num_of_chunk as u64 * chunk_size as u64;
    
    let mut buffer = device.create_buffer(upload_size, usage).unwrap();
    let mem_req = device.get_buffer_requirements(&buffer);
    let upload_type = memory_types
        .iter()
        .enumerate()
        .position(|(id, ty)| mem_req.type_mask & (1 << id) != 0 && ty.properties.contains(properties))
        .unwrap()
        .into();
    
    let memory = device.allocate_memory(upload_type, mem_req.size).unwrap();
    // TODO: error-check?
    device.bind_buffer_memory(&memory, 0, &mut buffer)
          .expect("failed to bind buffer memory");
    (buffer, memory, mem_req.size, chunk_size)
}

unsafe fn fill_buffer_with_alignment<I: Copy>(
    device: &<B as TB>::Device,
    buffer_memory: &mut <B as TB>::Memory,
    items: Vec<I>,
    num_of_chunk: u32,
    items_count_each_chunk: u32,
    row_pitch: u32,
    size: u64,
) {
    let mut data_target = device
        .acquire_mapping_writer::<I>(&buffer_memory, 0..size)
        .unwrap();
    let stride = ::std::mem::size_of::<I>() as usize;
    for current_chunk in 0..num_of_chunk as usize {
        let data_source_slice = &items
            [current_chunk * (items_count_each_chunk as usize) * stride
            ..(current_chunk + 1) * (items_count_each_chunk as usize) * stride];
        let dest_base = current_chunk * row_pitch as usize;
        
        data_target[dest_base..dest_base + data_source_slice.len()]
            .copy_from_slice(data_source_slice);
    }
    
    device.release_mapping_writer(data_target).unwrap();
}

