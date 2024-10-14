use std::marker::PhantomData;

use oxidx::dx::{self, IDevice};

use super::worker_type::WorkerType;

#[derive(Debug)]
pub(crate) struct CommandAllocator<T: WorkerType> {
    pub(crate) raw: dx::CommandAllocator,
    pub(crate) fence_value: u64,
    _marker: PhantomData<T>,
}

impl<T: WorkerType> CommandAllocator<T> {
    pub(crate) fn inner_new(device: &dx::Device, r#type: dx::CommandListType) -> Self {
        let raw = device.create_command_allocator(r#type).unwrap();

        Self {
            raw,
            fence_value: 0,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use crate::graphics::commands::worker_type::{Compute, Graphics, Transfer};

    use super::CommandAllocator;

    const fn is_send<T: Send>() {}

    const _: () = is_send::<CommandAllocator<Graphics>>();
    const _: () = is_send::<CommandAllocator<Compute>>();
    const _: () = is_send::<CommandAllocator<Transfer>>();
}
