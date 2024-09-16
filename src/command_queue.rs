#![allow(private_bounds)]

use std::{marker::PhantomData, ops::Deref, sync::Arc};

use oxidx::dx::{self, ICommandQueue, IDevice, PSO_NONE};
use parking_lot::Mutex;

use crate::{command_allocator::CommandAllocator, fence::FenceType, worker_thread::WorkerThread};

pub(super) trait WorkerType {}

pub(super) struct Graphics;
impl WorkerType for Graphics {}

pub(super) struct Compute;
impl WorkerType for Compute {}

pub(super) struct Transfer;
impl WorkerType for Transfer {}

#[derive(Clone)]
pub struct CommandQueue<T: WorkerType>(Arc<CommandQueueInner<T>>);

impl<T: WorkerType> Deref for CommandQueue<T> {
    type Target = CommandQueueInner<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct CommandQueueInner<T: WorkerType> {
    queue: Mutex<dx::CommandQueue>,

    cmd_allocators: Mutex<Vec<CommandAllocator<T>>>,
    cmd_list: Mutex<Vec<dx::GraphicsCommandList>>,

    pending_list: Mutex<Vec<WorkerThread<T>>>,
    temp_buffer: Mutex<Vec<Option<dx::GraphicsCommandList>>>,
    fence: FenceType,
    _marker: PhantomData<T>,
}

impl<T: WorkerType> CommandQueue<T> {
    fn inner_new(device: &dx::Device, fence: FenceType, desc: &dx::CommandQueueDesc) -> Self {
        let queue = device.create_command_queue(desc).unwrap();

        let cmd_allocators = (0..3)
            .map(|_| CommandAllocator::inner_new(device, desc.r#type()))
            .collect::<Vec<CommandAllocator<T>>>();

        let cmd_list = vec![device
            .create_command_list(0, desc.r#type(), &cmd_allocators[0].raw, PSO_NONE)
            .unwrap()];

        Self(Arc::new(CommandQueueInner {
            queue: Mutex::new(queue),

            cmd_allocators: Mutex::new(cmd_allocators),
            cmd_list: Mutex::new(cmd_list),

            pending_list: Default::default(),
            temp_buffer: Default::default(),
            fence,
            _marker: PhantomData,
        }))
    }
}

impl<T: WorkerType> CommandQueueInner<T> {
    pub fn push_fiber(&self, fiber: WorkerThread<T>) {
        self.temp_buffer.lock().push(Some(fiber.list.clone()));
        self.pending_list.lock().push(fiber);
    }

    pub fn execute(&self) {
        let threads = self.pending_list.lock().drain(..).collect::<Vec<_>>();

        let lists = self.temp_buffer.lock().drain(..).collect::<Vec<_>>();

        self.queue.lock().execute_command_lists(&lists);
    }
}

impl CommandQueue<Graphics> {
    pub fn graphics(device: &dx::Device, fence: FenceType) -> Self {
        Self::inner_new(device, fence, &dx::CommandQueueDesc::direct())
    }
}

impl CommandQueue<Compute> {
    pub fn compute(device: &dx::Device, fence: FenceType) -> Self {
        Self::inner_new(device, fence, &dx::CommandQueueDesc::compute())
    }
}

impl CommandQueue<Transfer> {
    pub fn transfer(device: &dx::Device, fence: FenceType) -> Self {
        Self::inner_new(device, fence, &dx::CommandQueueDesc::copy())
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandQueue, Compute, Graphics, Transfer};

    const fn is_send_sync<T: Send + Sync>() {}

    const _: () = is_send_sync::<CommandQueue<Graphics>>();
    const _: () = is_send_sync::<CommandQueue<Compute>>();
    const _: () = is_send_sync::<CommandQueue<Transfer>>();
}
