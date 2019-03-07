use std::{
    cell::RefCell,
    marker::PhantomData,
    rc::Rc,
};
use std::iter;

use super::{
    adapter::AdapterState,
    buffer::BufferState,
    descriptor::DescriptorState,
    device::DeviceState,
    prelude::*,
};

pub const BYTES_PIXEL: u32 = 4;
pub const COLOR_RANGE: SubresourceRange =
    SubresourceRange {
        aspects: Aspects::COLOR,
        levels: 0..1,
        layers: 0..1,
    };

pub struct ImageState {
    pub device_state: Rc<RefCell<DeviceState>>,
    pub image: Option<<B as TB>::Image>,
    pub memory: Option<<B as TB>::Memory>,
    pub image_view: Option<<B as TB>::ImageView>,
}

impl ImageState {
    pub fn new(
        device_state: Rc<RefCell<DeviceState>>,
        adapter_state: &AdapterState,
        width: u32,
        height: u32,
        color_format: Format,
    ) -> ImageState {
        let (image, memory, image_view) = unsafe {
            let device = &device_state.borrow_mut().device;
            let mut image = device.create_image(
                image::Kind::D2(width as image::Size, height as image::Size, 1, 1),
                1,
                color_format,
                image::Tiling::Optimal,
                image::Usage::TRANSFER_DST | image::Usage::SAMPLED,
                image::ViewCapabilities::empty(),
            ).expect("cannot create Image");
            
            let memory_requirements = device.get_image_requirements(&image);
            
            let device_type = adapter_state.choose_memory_type_from_memory_requirement(
                memory_requirements,
                Properties::DEVICE_LOCAL,
            );
            
            let memory =
                device
                    .allocate_memory(device_type, memory_requirements.size)
                    .unwrap();
            
            device
                .bind_image_memory(&memory, 0, &mut image)
                .unwrap();
            
            let image_view = device
                .create_image_view(
                    &image,
                    image::ViewKind::D2,
                    color_format,
                    Swizzle::NO,
                    COLOR_RANGE.clone(),
                ).unwrap();
            
            (Some(image), Some(memory), Some(image_view))
        };
        ImageState {
            device_state,
            image,
            memory,
            image_view,
        }
    }
}

pub struct SamplerState {
    pub device_state: Rc<RefCell<DeviceState>>,
    pub sampler: Option<<B as TB>::Sampler>,
}

impl SamplerState {
    pub fn new(device_state: Rc<RefCell<DeviceState>>, sampler_info: SamplerInfo) -> Self {
        let sampler = unsafe {
            let device = &device_state.borrow_mut().device;
            
            device.create_sampler(sampler_info)
        }.ok();
        SamplerState {
            device_state,
            sampler,
        }
    }
}

impl Drop for SamplerState {
    fn drop(&mut self) {
        unsafe {
            if let Some(sampler) = self.sampler.take() {
                let device = &self.device_state.borrow_mut().device;
                
                device.destroy_sampler(sampler);
            }
        }
    }
}

pub struct SampledImageState {
    pub device_state: Rc<RefCell<DeviceState>>,
    pub image_state: ImageState,
    pub buffer_state: BufferState<u8>,
    pub image_dimensions: (u32, u32),
    pub buffer_pitch_size: u32,
    pub sampler_state: SamplerState,
}

impl SampledImageState {
    pub fn new_from_image_buffer(
        device_state: Rc<RefCell<DeviceState>>,
        adapter_state: &AdapterState,
        image: ::image::ImageBuffer<::image::Rgba<u8>, Vec<u8>>,
    ) -> SampledImageState {
        let (width, height) = image.dimensions();
        let format = Format::Rgba32Sfloat;
        let (buffer_state, buffer_pitch_size) = BufferState::new_texture_buffer(
            device_state.clone(),
            adapter_state,
            width,
            height,
            BYTES_PIXEL,
            image.into_raw(),
            buffer::Usage::TRANSFER_SRC,
        );
        let image_state = ImageState::new(
            device_state.clone(),
            adapter_state,
            width,
            height,
            format,
        );
        
        let sampler_state = SamplerState::new(
            device_state.clone(),
            SamplerInfo::new(
                Filter::Linear,
                WrapMode::Clamp,
            ),
        );
        
        SampledImageState {
            device_state,
            image_state,
            buffer_state,
            image_dimensions: (width, height),
            buffer_pitch_size,
            sampler_state,
        }
    }
    pub fn register_descriptor(&mut self, descriptor_state: &DescriptorState) {
        unsafe {
            let device = &self.device_state.borrow().device;
            
            let set = descriptor_state.descriptor_set.as_ref().unwrap();
            device.write_descriptor_sets(
                vec![
                    DescriptorSetWrite {
                        set,
                        binding: 0,
                        array_offset: 0,
                        descriptors: &[Descriptor::Image(
                            self.image_state.image_view.as_ref().unwrap(),
                            Layout::Undefined,
                        )],
                    },
                    DescriptorSetWrite {
                        set,
                        binding: 1,
                        array_offset: 0,
                        descriptors: &[Descriptor::Sampler(
                            self.sampler_state.sampler.as_ref().unwrap()
                        )],
                        
                    }
                ]
            )
        }
    }
    
    pub unsafe fn transfer(
        &mut self,
        staging_pool: &mut CommandPool<B, Graphics>,
    ) -> CommandBuffer<B, Graphics> {
        let mut command_buffer = staging_pool.acquire_command_buffer::<OneShot>();
        command_buffer.begin();
        
        let image_barrier = Barrier::Image {
            states: (Access::empty(), Layout::Undefined)
                ..(Access::TRANSFER_WRITE, Layout::TransferDstOptimal),
            target: self.image_state.image.as_ref().unwrap(),
            families: None,
            range: COLOR_RANGE.clone(),
        };
        
        command_buffer.pipeline_barrier(
            PipelineStage::TOP_OF_PIPE..PipelineStage::TRANSFER,
            Dependencies::empty(),
            &[image_barrier],
        );
        
        command_buffer.copy_buffer_to_image(
            self.buffer_state.buffer.as_ref().unwrap(),
            self.image_state.image.as_ref().unwrap(),
            Layout::TransferDstOptimal,
            &[
                BufferImageCopy {
                    buffer_offset: 0,
                    buffer_width: self.buffer_pitch_size / BYTES_PIXEL as u32,
                    buffer_height: self.image_dimensions.1 as u32,
                    image_layers:
                    SubresourceLayers {
                        aspects: Aspects::COLOR,
                        level: 0,
                        layers: 0..1,
                    },
                    image_offset: Offset { x: 0, y: 0, z: 0 },
                    image_extent:
                    Extent {
                        width: self.image_dimensions.0,
                        height: self.image_dimensions.1,
                        depth: 1,
                    },
                }
            ],
        );
        
        let image_barrier = Barrier::Image {
            states: (Access::TRANSFER_WRITE, Layout::TransferDstOptimal)
                ..(Access::SHADER_READ, Layout::ShaderReadOnlyOptimal),
            target: self.image_state.image.as_ref().unwrap(),
            families: None,
            range: COLOR_RANGE.clone(),
        };
        command_buffer.pipeline_barrier(
            PipelineStage::TRANSFER..PipelineStage::FRAGMENT_SHADER,
            Dependencies::empty(),
            &[image_barrier],
        );
        
        command_buffer.finish();
        command_buffer
    }
    
    pub fn new(
        device_state: Rc<RefCell<DeviceState>>,
        adapter_state: &AdapterState,
        image: ::image::ImageBuffer<::image::Rgba<u8>, Vec<u8>>,
        usage: buffer::Usage,
    ) -> SampledImageState
    {
        let (buffer_state, dims, row_pitch, stride) = {
            let (width, height) = image.dimensions();
            
            let row_alignment_mask = adapter_state.limits.min_buffer_copy_pitch_alignment as u32 - 1;
            let stride = 4usize;
            
            let row_pitch = (width * stride as u32 + row_alignment_mask) & !row_alignment_mask;
            let upload_size = (height * row_pitch) as u64;
            
            let memory: <B as TB>::Memory;
            let mut buffer: <B as TB>::Buffer;
            let size: u64;
            
            unsafe {
                let device = &device_state.borrow().device;
                let adapter = &adapter_state.adapter;
                buffer = device.create_buffer(upload_size, usage).unwrap();
                let mem_reqs = device.get_buffer_requirements(&buffer);
                
                let upload_type = adapter_state
                    .memory_types
                    .iter()
                    .enumerate()
                    .position(|(id, mem_type)| {
                        mem_reqs.type_mask & (1 << id) != 0
                            && mem_type.properties.contains(Properties::CPU_VISIBLE)
                    })
                    .unwrap()
                    .into();
                
                memory = device.allocate_memory(upload_type, mem_reqs.size).unwrap();
                device.bind_buffer_memory(&memory, 0, &mut buffer).unwrap();
                size = mem_reqs.size;
                
                // copy image data into staging buffer
                {
                    let mut data_target = device
                        .acquire_mapping_writer::<u8>(&memory, 0..size)
                        .unwrap();
                    
                    for y in 0..height as usize {
                        let data_source_slice = &(*image)[y * (width as usize) * stride..(y + 1) *
                            (width as usize) * stride];
                        let dest_base = y * row_pitch as usize;
                        
                        data_target[dest_base..dest_base + data_source_slice.len()]
                            .copy_from_slice(data_source_slice);
                    }
                    
                    device.release_mapping_writer(data_target).unwrap();
                }
            }
            
            (
                BufferState {
                    memory: Some(memory),
                    buffer: Some(buffer),
                    size: Some(size),
                    device_state: device_state.clone(),
                    _phantom_data: PhantomData,
                },
                (width, height),
                row_pitch,
                stride,
            )
        };
        
        let device = &device_state.borrow().device;
        
        let kind = image::Kind::D2(dims.0 as image::Size, dims.1 as image::Size, 1, 1);
        let mut image = unsafe {
            device
                .create_image(
                    kind,
                    1,
                    Format::Rgba8Srgb,
                    image::Tiling::Optimal,
                    image::Usage::TRANSFER_DST | image::Usage::SAMPLED,
                    image::ViewCapabilities::empty(),
                )
        }.unwrap();
        let req = unsafe { device.get_image_requirements(&image) };
        
        let device_type = adapter_state
            .memory_types
            .iter()
            .enumerate()
            .position(|(id, memory_type)| {
                req.type_mask & (1 << id) != 0
                    && memory_type.properties.contains(Properties::DEVICE_LOCAL)
            })
            .unwrap()
            .into();
        
        let memory = unsafe { device.allocate_memory(device_type, req.size) }.unwrap();
        
        unsafe { device.bind_image_memory(&memory, 0, &mut image) }.unwrap();
        
        let image_view = unsafe {
            device
                .create_image_view(
                    &image,
                    image::ViewKind::D2,
                    Format::Rgba8Srgb,
                    Swizzle::NO,
                    COLOR_RANGE.clone(),
                )
        }.unwrap();
        
        let sampler = unsafe {
            device
                .create_sampler(
                    image::SamplerInfo::new(
                        image::Filter::Nearest,
                        image::WrapMode::Clamp,
                    )
                )
        }.expect("Can't create sampler");
        
        SampledImageState {
            device_state: device_state.clone(),
            image_state: ImageState {
                device_state: device_state.clone(),
                image: Some(image),
                memory: Some(memory),
                image_view: Some(image_view),
            },
            buffer_state,
            image_dimensions: dims,
            buffer_pitch_size: row_pitch,
            sampler_state: SamplerState {
                device_state: device_state.clone(),
                sampler: Some(sampler),
            },
        }
    }
}