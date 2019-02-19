use super::prelude::*;

pub struct DeviceState {
    pub device: <B as TB>::Device,
    pub queue_group: gfx_hal::QueueGroup<B, Graphics>,
    pub command_pool: Option<gfx_hal::CommandPool<B, Graphics>>,
}

impl DeviceState {
    pub fn new(
        adapter: &Adapter<B>,
        surface: &<B as TB>::Surface,
    ) -> Self {
        let (device, queue_group) = adapter
            .open_with::<_, Graphics>(
                1,
                |family| surface.supports_queue_family(family))
            .unwrap();
        let mut command_pool =
            unsafe {
                device.create_command_pool_typed(&queue_group, CommandPoolCreateFlags::empty())
            }.unwrap();
        DeviceState {
            device,
            queue_group,
            command_pool: Some(command_pool),
        }
    }
}

impl Drop for DeviceState {
    fn drop(&mut self) {
        let device = &self.device;
        unsafe {
            device.destroy_command_pool(self.command_pool.take().unwrap().into_raw());
        }
    }
}