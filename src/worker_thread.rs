#![allow(private_bounds)]

use crate::{
    command_queue::{WorkerType, Compute, Copy, Graphics},
    frame_command_allocator::FrameCommandAllocator,
};

use oxidx::dx::{self, ICommandAllocator, IDevice, IGraphicsCommandList};

pub struct WorkerThread<T: WorkerType> {
    pub(super) list: dx::GraphicsCommandList,
    allocator: FrameCommandAllocator<T>,
}

impl<T: WorkerType> WorkerThread<T> {
    fn inner_new(
        device: &dx::Device,
        allocator: FrameCommandAllocator<T>,
        r#type: dx::CommandListType,
    ) -> Self {
        let list = device
            .create_command_list(0, r#type, &allocator.inner[0], dx::PSO_NONE)
            .unwrap();

        Self {
            list,
            allocator
        }
    }

    pub fn reset(&mut self, pso: Option<&dx::PipelineState>) {
        let allocator = self.allocator.next_allocator();
        allocator.reset().unwrap();
        self.list.reset(allocator, pso).unwrap();
    }

    pub fn close(&self) {
        self.list.close().unwrap();
    }
}

impl WorkerThread<Graphics> {
    pub fn graphics(device: &dx::Device, allocator: FrameCommandAllocator<Graphics>) -> Self {
        Self::inner_new(
            device,
            allocator,
            dx::CommandListType::Direct,
        )
    }
}

impl WorkerThread<Compute> {
    pub fn compute(device: &dx::Device, allocator: FrameCommandAllocator<Compute>) -> Self {
        Self::inner_new(
            device,
            allocator,
            dx::CommandListType::Compute,
        )
    }
}

impl WorkerThread<Copy> {
    pub fn copy(device: &dx::Device, allocator: FrameCommandAllocator<Copy>) -> Self {
        Self::inner_new(
            device,
            allocator,
            dx::CommandListType::Copy,
        )
    }
}
