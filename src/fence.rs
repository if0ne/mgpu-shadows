use std::sync::atomic::AtomicU64;

use oxidx::dx::{self, IDevice};

pub enum FenceType {
    Local(LocalFence),
    Shared(SharedFence),
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
    owner: dx::Device,
    pub(super) fences: Vec<dx::Fence>,
    value: AtomicU64,
}

impl SharedFence {
    pub fn new<'a>(owner: dx::Device) -> Self {
        let fence = owner
            .create_fence(
                0,
                dx::FenceFlags::Shared | dx::FenceFlags::SharedCrossAdapter,
            )
            .unwrap();

        Self {
            owner,
            fences: vec![fence],
            value: AtomicU64::default(),
        }
    }

    pub fn add_device(&mut self, device: &dx::Device) {
        let handle = self
            .owner
            .create_shared_handle(&self.fences[0].clone().into(), None)
            .unwrap();
        let fence = device.open_shared_handle(handle).unwrap();
        self.fences.push(fence);
        handle.close().unwrap();
    }
}
