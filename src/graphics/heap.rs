use oxidx::dx::{self, ClearValue, IDevice};

use super::{device::Device, resources::SharedResource};

#[derive(Clone)]
pub struct SharedHeap {
    owner: Device,
    heap: dx::Heap,
}

impl SharedHeap {
    pub(super) fn inner_new(owner: Device, size: usize) -> Self {
        let heap = owner
            .raw
            .create_heap(
                &dx::HeapDesc::new(size, dx::HeapProperties::default())
                    .with_flags(dx::HeapFlags::Shared | dx::HeapFlags::SharedCrossAdapter),
            )
            .unwrap();

        Self { owner, heap }
    }
}

impl SharedHeap {
    pub fn connect(&self, device: Device) -> Self {
        let handle = self
            .owner
            .raw
            .create_shared_handle(&self.heap, None)
            .unwrap();
        let heap = device.raw.open_shared_handle(handle).unwrap();
        handle.close().unwrap();

        Self {
            owner: device,
            heap,
        }
    }

    pub fn heap(&self) -> &dx::Heap {
        &self.heap
    }

    pub fn device(&self) -> &Device {
        &self.owner
    }

    pub fn create_shared_resource(
        &self,
        offset: usize,
        desc: dx::ResourceDesc,
        local_state: dx::ResourceStates,
        share_state: dx::ResourceStates,
        clear_color: Option<&ClearValue>,
    ) -> SharedResource {
        SharedResource::inner_new(self, offset, desc, local_state, share_state, clear_color)
    }
}
