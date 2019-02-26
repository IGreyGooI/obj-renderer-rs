use crate::frontend::graphic::hal::adapter::AdapterState;

use super::prelude::*;

pub struct DeviceState {
    pub device: <B as TB>::Device,
    pub queue_group: gfx_hal::QueueGroup<B, Graphics>,
}

impl DeviceState {
    pub fn new(
        adapter_state: &AdapterState,
        surface: &<B as TB>::Surface,
    ) -> Self {
        let (device, queue_group) = adapter_state.adapter
                                                 .open_with::<_, Graphics>(
                1,
                |family| surface.supports_queue_family(family))
                                                 .unwrap();
    
        DeviceState {
            device,
            queue_group,
        }
    }
}
