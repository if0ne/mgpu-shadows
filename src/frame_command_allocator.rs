#![allow(private_bounds)]

use std::marker::PhantomData;

use oxidx::dx::{self, IDevice};

use crate::command_queue::{CommandType, Compute, Copy, Graphics};

pub struct FrameCommandAllocator<T: CommandType> {
    inner: [dx::CommandAllocator; 4],
    cur: usize,
    _marker: PhantomData<T>,
}

impl<T: CommandType> FrameCommandAllocator<T> {
    fn inner_new(device: &dx::Device, r#type: dx::CommandListType) -> Self {
        let inner = std::array::from_fn(|_| device.create_command_allocator(r#type).unwrap());

        Self {
            inner,
            cur: 0,
            _marker: PhantomData,
        }
    }

    pub fn next_allocator(&mut self) -> &dx::CommandAllocator {
        let old = self.cur;
        self.cur = (self.cur + 1) % 4;

        &self.inner[old]
    }
}

impl FrameCommandAllocator<Graphics> {
    pub fn graphics(device: &dx::Device) -> Self {
        Self::inner_new(device, dx::CommandListType::Direct)
    }
}

impl FrameCommandAllocator<Compute> {
    pub fn compute(device: &dx::Device) -> Self {
        Self::inner_new(device, dx::CommandListType::Compute)
    }
}

impl FrameCommandAllocator<Copy> {
    pub fn copy(device: &dx::Device) -> Self {
        Self::inner_new(device, dx::CommandListType::Copy)
    }
}
