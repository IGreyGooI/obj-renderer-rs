use super::prelude::*;

/// a struct contain Adapter
/// and caching some information of Adapter, which is good for later use
pub struct AdapterState {
    pub adapter: Adapter<B>,
    pub memory_types: Vec<MemoryType>,
    pub limits: Limits,
}

impl AdapterState {
    pub fn new(adapters: &mut Vec<Adapter<B>>) -> AdapterState {
        for adapter in adapters.iter() {
            println!("[INFO][Adapter Detected]{:?}", adapter.info);
        }
        
        AdapterState::new_from_adapter(adapters.remove(0))
    }
    
    fn new_from_adapter(adapter: Adapter<B>) -> AdapterState {
        println!("[INFO][Adapter Chosen]{:?}", adapter.info);
        let physical_device = &adapter.physical_device;
        let memory_types = physical_device.memory_properties().memory_types;
        println!("[INFO][Adapter Memory Types Available]{:?}", memory_types);
        let limits = physical_device.limits();
        println!("[INFO][Adapter Limits]{:?}", limits);
        
        AdapterState {
            adapter,
            memory_types,
            limits,
        }
    }
    pub fn choose_memory_type_from_memory_requirement(
        &self,
        memory_requirements: Requirements,
        properties: Properties,
    ) -> MemoryTypeId {
        self.memory_types
            .iter()
            .enumerate()
            .position(|(id, memory_type)| {
                memory_requirements.type_mask & (1 << id) != 0
                    && memory_type.properties.contains(properties)
            })
            .unwrap()
            .into()
    }
}

/// containing the logic on how select a Adapter
pub fn select_adapter(adapters: &mut Vec<Adapter<B>>) -> Adapter<B> {
    adapters.remove(0)
}