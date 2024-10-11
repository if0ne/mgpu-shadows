use std::sync::{atomic::AtomicU64, Arc};

use oxidx::dx::{self, IDevice, IFence};

use super::device::Device;

#[derive(Debug, Clone)]
pub enum Fence {
    Local(LocalFence),
    Shared(SharedFence),
}

impl Fence {
    pub fn get_completed_value(&self) -> u64 {
        match self {
            Fence::Local(fence) => fence.get_completed_value(),
            Fence::Shared(fence) => fence.get_completed_value(),
        }
    }

    pub fn set_event_on_completion(&self, value: u64, event: dx::Event) {
        match self {
            Fence::Local(fence) => fence.set_event_on_completion(value, event),
            Fence::Shared(fence) => fence.set_event_on_completion(value, event),
        }
    }

    pub fn inc_value(&self) -> u64 {
        match self {
            Fence::Local(fence) => fence.inc_value(),
            Fence::Shared(fence) => fence.inc_value(),
        }
    }

    pub fn get_current_value(&self) -> u64 {
        match self {
            Fence::Local(fence) => fence.get_current_value(),
            Fence::Shared(fence) => fence.get_current_value(),
        }
    }

    pub(super) fn get_raw(&self) -> &dx::Fence {
        match self {
            Fence::Local(fence) => fence.get_raw(),
            Fence::Shared(fence) => fence.get_raw(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LocalFence {
    pub(super) raw: dx::Fence,
    value: Arc<AtomicU64>,
}

impl LocalFence {
    pub(super) fn inner_new(device: &Device) -> Self {
        let fence = device.raw.create_fence(0, dx::FenceFlags::empty()).unwrap();

        Self {
            raw: fence,
            value: Default::default(),
        }
    }

    pub(super) fn get_raw(&self) -> &dx::Fence {
        &self.raw
    }

    pub fn get_completed_value(&self) -> u64 {
        self.raw.get_completed_value()
    }

    pub fn set_event_on_completion(&self, value: u64, event: dx::Event) {
        self.raw.set_event_on_completion(value, event).unwrap();
    }

    pub fn inc_value(&self) -> u64 {
        self.value
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1
    }

    pub fn get_current_value(&self) -> u64 {
        self.value.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl From<LocalFence> for Fence {
    fn from(value: LocalFence) -> Self {
        Fence::Local(value)
    }
}

#[derive(Clone, Debug)]
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

    pub(super) fn get_raw(&self) -> &dx::Fence {
        &self.fence
    }

    pub fn get_completed_value(&self) -> u64 {
        self.fence.get_completed_value()
    }

    pub fn set_event_on_completion(&self, value: u64, event: dx::Event) {
        self.fence.set_event_on_completion(value, event).unwrap();
    }

    pub fn inc_value(&self) -> u64 {
        self.value
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1
    }

    pub fn get_current_value(&self) -> u64 {
        self.value.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl SharedFence {
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

impl From<SharedFence> for Fence {
    fn from(value: SharedFence) -> Self {
        Fence::Shared(value)
    }
}
