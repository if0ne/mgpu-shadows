use std::sync::{atomic::AtomicU64, Arc};

use oxidx::dx::{self, IDevice, IFence};

pub trait Fence {
    fn get_completed_value(&self) -> u64;
    fn set_event_on_completion(&self, value: u64, event: dx::Event);

    fn inc_fence_value(&self) -> u64;

    fn get_raw(&self) -> &dx::Fence;
}

impl Fence for LocalFence {
    fn get_completed_value(&self) -> u64 {
        self.raw.get_completed_value()
    }

    fn set_event_on_completion(&self, value: u64, event: dx::Event) {
        self.raw.set_event_on_completion(value, event).unwrap();
    }

    fn inc_fence_value(&self) -> u64 {
        self.value
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1
    }

    fn get_raw(&self) -> &dx::Fence {
        &self.raw
    }
}

pub struct LocalFence {
    pub(super) raw: dx::Fence,
    value: AtomicU64,
}

impl LocalFence {
    pub fn new(device: &dx::Device) -> Self {
        let fence = device.create_fence(0, dx::FenceFlags::empty()).unwrap();

        Self {
            raw: fence,
            value: AtomicU64::default(),
        }
    }
}

pub struct SharedFence {
    inner: Arc<SharedFenceInner>,
    state: SharedFenceState,
}

impl SharedFence {
    pub fn new(owner: dx::Device) -> Self {
        let owner_fence = owner
            .create_fence(
                0,
                dx::FenceFlags::Shared | dx::FenceFlags::SharedCrossAdapter,
            )
            .unwrap();

        Self {
            inner: Arc::new(SharedFenceInner {
                owner,
                owner_fence,
                value: Default::default(),
            }),
            state: SharedFenceState::Owner,
        }
    }

    pub fn connect(&mut self, device: &dx::Device) -> SharedFence {
        let handle = self
            .inner
            .owner
            .create_shared_handle(&self.inner.owner_fence.clone().into(), None)
            .unwrap();
        let fence = device.open_shared_handle(handle).unwrap();
        handle.close().unwrap();

        Self {
            inner: Arc::clone(&self.inner),
            state: SharedFenceState::Connected { fence },
        }
    }
}

struct SharedFenceInner {
    owner: dx::Device,
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
            SharedFenceState::Owner => self.inner.owner_fence.get_completed_value(),
            SharedFenceState::Connected { fence } => fence.get_completed_value(),
        }
    }

    fn set_event_on_completion(&self, value: u64, event: dx::Event) {
        match &self.state {
            SharedFenceState::Owner => self
                .inner
                .owner_fence
                .set_event_on_completion(value, event)
                .unwrap(),
            SharedFenceState::Connected { fence } => {
                fence.set_event_on_completion(value, event).unwrap()
            }
        };
    }

    fn inc_fence_value(&self) -> u64 {
        self.inner
            .value
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1
    }

    fn get_raw(&self) -> &dx::Fence {
        match &self.state {
            SharedFenceState::Owner => &self.inner.owner_fence,
            SharedFenceState::Connected { fence } => fence,
        }
    }
}
