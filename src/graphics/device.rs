#![allow(private_bounds)]

use std::{ops::Deref, sync::Arc};

use oxidx::dx;

use super::{
    command_allocator::CommandAllocator,
    command_queue::{CommandQueue, Compute, Graphics, Transfer, WorkerType},
    fence::Fence,
};

#[derive(Clone)]
pub struct Device(Arc<DeviceInner>);

impl Deref for Device {
    type Target = DeviceInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct DeviceInner {
    name: String,
    adapter: dx::Adapter3,
    pub(super) raw: dx::Device,
}

impl Device {
    pub(super) fn create_command_allocator<T: WorkerType>(&self) -> CommandAllocator<T> {
        CommandAllocator::inner_new(&self.raw, T::RAW_TYPE)
    }
}

impl Device {
    pub fn create_graphics_command_queue<F: Fence>(&self, fence: F) -> CommandQueue<Graphics, F> {
        CommandQueue::inner_new(self.clone(), fence, &dx::CommandQueueDesc::direct())
    }

    pub fn create_compute_command_queue<F: Fence>(&self, fence: F) -> CommandQueue<Compute, F> {
        CommandQueue::inner_new(self.clone(), fence, &dx::CommandQueueDesc::direct())
    }

    pub fn create_transfer_command_queue<F: Fence>(&self, fence: F) -> CommandQueue<Transfer, F> {
        CommandQueue::inner_new(self.clone(), fence, &dx::CommandQueueDesc::direct())
    }
}
