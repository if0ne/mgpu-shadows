use std::sync::{atomic::AtomicU64, Arc};

use oxidx::dx::{self, IDevice, IFence};

use super::device::Device;

pub trait Fence {
    fn get_completed_value(&self) -> u64;
    fn set_event_on_completion(&self, value: u64, event: dx::Event);

    fn inc_value(&self) -> u64;
    fn get_current_value(&self) -> u64;

    fn get_raw(&self) -> &dx::Fence;
}

impl Fence for LocalFence {
    fn get_completed_value(&self) -> u64 {
        self.raw.get_completed_value()
    }

    fn set_event_on_completion(&self, value: u64, event: dx::Event) {
        self.raw.set_event_on_completion(value, event).unwrap();
    }

    fn inc_value(&self) -> u64 {
        self.value
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1
    }

    fn get_raw(&self) -> &dx::Fence {
        &self.raw
    }

    fn get_current_value(&self) -> u64 {
        self.value.load(std::sync::atomic::Ordering::Relaxed)
    }
}

pub struct LocalFence {
    pub(super) raw: dx::Fence,
    value: AtomicU64,
}

impl LocalFence {
    pub(super) fn inner_new(device: &Device) -> Self {
        let fence = device.raw.create_fence(0, dx::FenceFlags::empty()).unwrap();

        Self {
            raw: fence,
            value: AtomicU64::default(),
        }
    }
}

#[derive(Clone)]
pub struct SharedFence {
    owner: Device,
    fence: dx::Fence,
    value: Arc<AtomicU64>,
}

impl SharedFence {
    pub(super) fn inner_new(owner: Device) -> Self {
        let fence = owner
            .raw
            .create_fence(
                0,
                dx::FenceFlags::Shared | dx::FenceFlags::SharedCrossAdapter,
            )
            .unwrap();

        Self {
            owner,
            fence,
            value: Default::default(),
        }
    }

    pub fn connect(&self, device: Device) -> Self {
        let handle = self
            .owner
            .raw
            .create_shared_handle(&self.fence, None)
            .unwrap();
        let fence = device.raw.open_shared_handle(handle).unwrap();
        handle.close().unwrap();

        Self {
            owner: device,
            fence,
            value: Arc::clone(&self.value),
        }
    }
}

impl Fence for SharedFence {
    fn get_completed_value(&self) -> u64 {
        self.fence.get_completed_value()
    }

    fn set_event_on_completion(&self, value: u64, event: dx::Event) {
        self.fence.set_event_on_completion(value, event).unwrap()
    }

    fn inc_value(&self) -> u64 {
        self.value
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1
    }

    fn get_raw(&self) -> &dx::Fence {
        &self.fence
    }

    fn get_current_value(&self) -> u64 {
        self.value.load(std::sync::atomic::Ordering::Relaxed)
    }
}
