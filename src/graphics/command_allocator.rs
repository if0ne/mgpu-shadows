use std::marker::PhantomData;

use oxidx::dx::{self, ICommandAllocator, IDevice};

use super::command_queue::WorkerType;

#[derive(Debug)]
pub(super) struct CommandAllocator<T: WorkerType> {
    pub(super) raw: dx::CommandAllocator,
    pub(super) fence_value: u64,
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

    pub(super) fn reset(&self) {
        self.raw.reset().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::CommandAllocator;
    use crate::graphics::command_queue::Graphics;

    const fn is_send<T: Send>() {}

    const _: () = is_send::<CommandAllocator<Graphics>>();
}
