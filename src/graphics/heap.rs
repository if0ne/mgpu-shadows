use std::sync::Arc;

use oxidx::dx::{self, IDevice};

use super::{device::Device, resources::SharedResource};

pub struct SharedHeap {
    shared: Arc<SharedHeapInner>,
    state: SharedHeapState,
}

impl SharedHeap {
    pub(super) fn inner_new(owner: Device, size: usize) -> Self {
        let owner_heap = owner
            .raw
            .create_heap(
                &dx::HeapDesc::new(size, dx::HeapProperties::default())
                    .with_flags(dx::HeapFlags::Shared | dx::HeapFlags::SharedCrossAdapter),
            )
            .unwrap();

        Self {
            shared: Arc::new(SharedHeapInner { owner, owner_heap }),
            state: SharedHeapState::Owner,
        }
    }

    pub fn connect(&self, device: Device) -> Self {
        let handle = self
            .shared
            .owner
            .raw
            .create_shared_handle(&self.shared.owner_heap, None)
            .unwrap();
        let heap = device.raw.open_shared_handle(handle).unwrap();
        handle.close().unwrap();

        Self {
            shared: Arc::clone(&self.shared),
            state: SharedHeapState::Connected { heap, device },
        }
    }

    pub fn heap(&self) -> &dx::Heap {
        match &self.state {
            SharedHeapState::Owner => &self.shared.owner_heap,
            SharedHeapState::Connected { heap, .. } => heap,
        }
    }

    pub fn device(&self) -> &Device {
        match &self.state {
            SharedHeapState::Owner => &self.shared.owner,
            SharedHeapState::Connected { device, .. } => &device,
        }
    }

    pub fn create_shared_resource(&self, offset: usize, desc: &dx::ResourceDesc) -> SharedResource {
        SharedResource::inner_new(self, offset, desc)
    }
}

struct SharedHeapInner {
    pub(super) owner: Device,
    pub(super) owner_heap: dx::Heap,
}

enum SharedHeapState {
    Owner,
    Connected { device: Device, heap: dx::Heap },
}
