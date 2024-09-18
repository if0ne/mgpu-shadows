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

pub struct SharedFence {
    shared: Arc<SharedFenceInner>,
    state: SharedFenceState,
}

impl SharedFence {
    pub(super) fn inner_new(owner: Device) -> Self {
        let owner_fence = owner
            .raw
            .create_fence(
                0,
                dx::FenceFlags::Shared | dx::FenceFlags::SharedCrossAdapter,
            )
            .unwrap();

        Self {
            shared: Arc::new(SharedFenceInner {
                owner,
                owner_fence,
                value: Default::default(),
            }),
            state: SharedFenceState::Owner,
        }
    }

    pub fn connect(&mut self, device: &Device) -> Self {
        let handle = self
            .shared
            .owner
            .raw
            .create_shared_handle(&self.shared.owner_fence, None)
            .unwrap();
        let fence = device.raw.open_shared_handle(handle).unwrap();
        handle.close().unwrap();

        Self {
            shared: Arc::clone(&self.shared),
            state: SharedFenceState::Connected { fence },
        }
    }
}

struct SharedFenceInner {
    owner: Device,
    owner_fence: dx::Fence,
    value: AtomicU64,
}

enum SharedFenceState {
    Owner,
    Connected { fence: dx::Fence },
}

impl Fence for SharedFence {
    fn get_completed_value(&self) -> u64 {
        match &self.state {
            SharedFenceState::Owner => self.shared.owner_fence.get_completed_value(),
            SharedFenceState::Connected { fence } => fence.get_completed_value(),
        }
    }

    fn set_event_on_completion(&self, value: u64, event: dx::Event) {
        match &self.state {
            SharedFenceState::Owner => self
                .shared
                .owner_fence
                .set_event_on_completion(value, event)
                .unwrap(),
            SharedFenceState::Connected { fence } => {
                fence.set_event_on_completion(value, event).unwrap()
            }
        };
    }

    fn inc_value(&self) -> u64 {
        self.shared
            .value
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1
    }

    fn get_raw(&self) -> &dx::Fence {
        match &self.state {
            SharedFenceState::Owner => &self.shared.owner_fence,
            SharedFenceState::Connected { fence } => fence,
        }
    }

    fn get_current_value(&self) -> u64 {
        self.shared.value.load(std::sync::atomic::Ordering::Relaxed)
    }
}
