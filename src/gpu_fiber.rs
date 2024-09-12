#![allow(private_bounds)]

use std::marker::PhantomData;

use crate::frame_command_allocator::{
    CommandListType, Compute, Copy, FrameCommandAllocator, Graphics,
};

use oxidx::dx::{self, IDevice};

pub struct GpuFiber<T: CommandListType> {
    list: dx::GraphicsCommandList,
    _marker: PhantomData<T>,
}

impl<T: CommandListType> GpuFiber<T> {
    fn inner_new(
        device: &dx::Device,
        allocator: &dx::CommandAllocator,
        r#type: dx::CommandListType,
    ) -> Self {
        let list = device
            .create_command_list(0, r#type, allocator, dx::PSO_NONE)
            .unwrap();

        Self {
            list,
            _marker: PhantomData,
        }
    }
}

impl GpuFiber<Graphics> {
    pub fn graphics(device: &dx::Device, allocator: &mut FrameCommandAllocator<Graphics>) -> Self {
        Self::inner_new(
            device,
            allocator.next_allocator(),
            dx::CommandListType::Direct,
        )
    }
}

impl GpuFiber<Compute> {
    pub fn compute(device: &dx::Device, allocator: &mut FrameCommandAllocator<Compute>) -> Self {
        Self::inner_new(
            device,
            allocator.next_allocator(),
            dx::CommandListType::Compute,
        )
    }
}

impl GpuFiber<Copy> {
    pub fn copy(device: &dx::Device, allocator: &mut FrameCommandAllocator<Copy>) -> Self {
        Self::inner_new(
            device,
            allocator.next_allocator(),
            dx::CommandListType::Copy,
        )
    }
}
