#![allow(private_bounds)]

use crate::{
    command_queue::{CommandType, Compute, Copy, Graphics},
    frame_command_allocator::FrameCommandAllocator,
};

use oxidx::dx::{self, ICommandAllocator, IDevice, IGraphicsCommandList};

pub struct GpuFiber<T: CommandType> {
    pub(super) list: dx::GraphicsCommandList,
    allocator: FrameCommandAllocator<T>,
}

impl<T: CommandType> GpuFiber<T> {
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

impl GpuFiber<Graphics> {
    pub fn graphics(device: &dx::Device, allocator: FrameCommandAllocator<Graphics>) -> Self {
        Self::inner_new(
            device,
            allocator,
            dx::CommandListType::Direct,
        )
    }
}

impl GpuFiber<Compute> {
    pub fn compute(device: &dx::Device, allocator: FrameCommandAllocator<Compute>) -> Self {
        Self::inner_new(
            device,
            allocator,
            dx::CommandListType::Compute,
        )
    }
}

impl GpuFiber<Copy> {
    pub fn copy(device: &dx::Device, allocator: FrameCommandAllocator<Copy>) -> Self {
        Self::inner_new(
            device,
            allocator,
            dx::CommandListType::Copy,
        )
    }
}
