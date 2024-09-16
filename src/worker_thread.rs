#![allow(private_bounds)]

use crate::{
    command_allocator::CommandAllocator,
    command_queue::{Compute, Graphics, Transfer, WorkerType},
};

use oxidx::dx::{self, IDevice, IGraphicsCommandList};

pub struct WorkerThread<T: WorkerType> {
    pub(super) allocator: CommandAllocator<T>,
    pub(super) list: dx::GraphicsCommandList,
}

impl<T: WorkerType> WorkerThread<T> {
    fn inner_new(
        device: &dx::Device,
        allocator: CommandAllocator<T>,
        r#type: dx::CommandListType,
    ) -> Self {
        let list = device
            .create_command_list(0, r#type, &allocator.raw, dx::PSO_NONE)
            .unwrap();

        Self { list, allocator }
    }

    pub fn close(&self) {
        self.list.close().unwrap();
    }
}

impl WorkerThread<Graphics> {
    pub fn graphics(device: &dx::Device, allocator: CommandAllocator<Graphics>) -> Self {
        Self::inner_new(device, allocator, dx::CommandListType::Direct)
    }
}

impl WorkerThread<Compute> {
    pub fn compute(device: &dx::Device, allocator: CommandAllocator<Compute>) -> Self {
        Self::inner_new(device, allocator, dx::CommandListType::Compute)
    }
}

impl WorkerThread<Transfer> {
    pub fn transfer(device: &dx::Device, allocator: CommandAllocator<Transfer>) -> Self {
        Self::inner_new(device, allocator, dx::CommandListType::Copy)
    }
}
