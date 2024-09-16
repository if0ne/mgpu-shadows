#![allow(private_bounds)]

use std::marker::PhantomData;

use oxidx::dx::{self, ICommandAllocator, IDevice};

use crate::command_queue::{Compute, Graphics, Transfer, WorkerType};

pub struct CommandAllocator<T: WorkerType> {
    pub(super) raw: dx::CommandAllocator,
    fence_value: u64,
    _marker: PhantomData<T>,
}

impl<T: WorkerType> CommandAllocator<T> {
    pub(super) fn inner_new(device: &dx::Device, r#type: dx::CommandListType) -> Self {
        let raw = device.create_command_allocator(r#type).unwrap();

        Self {
            raw,
            fence_value: 0,
            _marker: PhantomData,
        }
    }

    pub(super) fn fence_value(&mut self) -> u64 {
        self.fence_value
    }

    pub(super) fn inc_fence_value(&mut self) {
        self.fence_value += 1;
    }

    pub(super) fn reset(&self) {
        self.raw.reset().unwrap();
    }
}

impl CommandAllocator<Graphics> {
    pub fn graphics(device: &dx::Device) -> Self {
        Self::inner_new(device, dx::CommandListType::Direct)
    }
}

impl CommandAllocator<Compute> {
    pub fn compute(device: &dx::Device) -> Self {
        Self::inner_new(device, dx::CommandListType::Compute)
    }
}

impl CommandAllocator<Transfer> {
    pub fn transfer(device: &dx::Device) -> Self {
        Self::inner_new(device, dx::CommandListType::Copy)
    }
}
