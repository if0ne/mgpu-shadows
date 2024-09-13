use oxidx::dx::{self, IDevice};

pub struct FrameFence {
    pub(super) inner: dx::Fence,
    values: [u64; 4],
    cur: usize,
}

impl FrameFence {
    pub fn new(device: &dx::Device) -> Self {
        let fence = device.create_fence(0, dx::FenceFlags::empty()).unwrap();

        Self {
            inner: fence,
            values: [0; 4],
            cur: 0,
        }
    }
}

pub struct SharedFrameFence {
    owner: dx::Device,
    pub(super) fences: Vec<dx::Fence>,
    values: [u64; 4],
    cur: usize,
}

impl SharedFrameFence {
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
            values: [0; 4],
            cur: 0,
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
