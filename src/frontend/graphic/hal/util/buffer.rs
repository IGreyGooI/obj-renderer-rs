use backend::Backend as B;
use gfx_hal::{
    buffer,
    memory::Properties,
    Backend as TB,
    device::Device,
    MemoryType,
};

pub unsafe fn create_buffer<I: Copy>(
    device: &<B as TB>::Device,
    memory_types: Vec<MemoryType>,
    properties: Properties,
    usage: buffer::Usage,
    items: Vec<I>,
) -> (<B as TB>::Buffer, <B as TB>::Memory) {
    let (buffer, mut memory) = create_empty_buffer::<I>(
        device,
        memory_types,
        properties,
        usage,
        items.len(),
    );
    
    fill_buffer(
        device,
        &mut memory,
        items,
    );
    (buffer, memory)
}

pub unsafe fn create_empty_buffer<I>(
    device: &<B as TB>::Device,
    memory_types: Vec<MemoryType>,
    properties: Properties,
    usage: buffer::Usage,
    item_count: usize,
) -> (<B as TB>::Buffer, <B as TB>::Memory) {
    let stride = ::std::mem::size_of::<I>() as u64;
    let buffer_len = item_count as u64 * stride;
    let mut buffer = device.create_buffer(buffer_len, usage).unwrap();
    let req = device.get_buffer_requirements(&buffer);
    let upload_type = memory_types
        .iter()
        .enumerate()
        .position(|(id, ty)| req.type_mask & (1 << id) != 0 && ty.properties.contains(properties))
        .unwrap()
        .into();
    let memory = device.allocate_memory(upload_type, req.size).unwrap();
    // TODO: error-check?
    device.bind_buffer_memory(&memory, 0, &mut buffer);
    (buffer, memory)
}

/// Pushes data into a buffer.
pub unsafe fn fill_buffer<I: Copy>(
    device: &<B as TB>::Device,
    buffer_memory: &mut <B as TB>::Memory,
    items: Vec<I>,
) {
    // NOTE: MESH -> items
    // NOTE: Recalc buffer_len
    
    let stride = ::std::mem::size_of::<I>() as u64;
    let buffer_len = items.len() as u64 * stride;
    
    let mut dest = device
        .acquire_mapping_writer::<I>(&buffer_memory, 0..buffer_len)
        .unwrap();
    
    dest.copy_from_slice(items.as_slice());
    device.release_mapping_writer(dest);
}

