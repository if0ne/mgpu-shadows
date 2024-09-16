#![allow(private_bounds)]

use std::{collections::VecDeque, marker::PhantomData, ops::Deref, sync::Arc};

use oxidx::dx::{self, ICommandQueue, IDevice, PSO_NONE};
use parking_lot::Mutex;

use crate::{command_allocator::CommandAllocator, fence::Fence, worker_thread::WorkerThread};

pub(super) trait WorkerType {}

pub(super) struct Graphics;
impl WorkerType for Graphics {}

pub(super) struct Compute;
impl WorkerType for Compute {}

pub(super) struct Transfer;
impl WorkerType for Transfer {}

#[derive(Clone)]
pub struct CommandQueue<T: WorkerType, F: Fence>(Arc<CommandQueueInner<T, F>>);

impl<T: WorkerType, F: Fence> Deref for CommandQueue<T, F> {
    type Target = CommandQueueInner<T, F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct CommandQueueInner<T: WorkerType, F: Fence> {
    device: dx::Device,

    queue: Mutex<dx::CommandQueue>,

    cmd_allocators: Mutex<VecDeque<CommandAllocator<T>>>,
    cmd_list: Mutex<Vec<dx::GraphicsCommandList>>,

    pending_list: Mutex<Vec<WorkerThread<T>>>,
    temp_buffer: Mutex<Vec<Option<dx::GraphicsCommandList>>>,
    fence: F,
    _marker: PhantomData<T>,
}

impl<T: WorkerType, F: Fence> CommandQueue<T, F> {
    fn inner_new(device: dx::Device, fence: F, desc: &dx::CommandQueueDesc) -> Self {
        let queue = device.create_command_queue(desc).unwrap();

        let cmd_allocators = (0..3)
            .map(|_| CommandAllocator::inner_new(&device, desc.r#type()))
            .collect::<VecDeque<CommandAllocator<T>>>();

        let cmd_list = vec![device
            .create_command_list(0, desc.r#type(), &cmd_allocators[0].raw, PSO_NONE)
            .unwrap()];

        Self(Arc::new(CommandQueueInner {
            device,
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

impl<T: WorkerType, F: Fence> CommandQueueInner<T, F> {
    pub fn push_fiber(&self, fiber: WorkerThread<T>) {
        self.temp_buffer.lock().push(Some(fiber.list.clone()));
        self.pending_list.lock().push(fiber);
    }

    fn signal(&self) -> u64 {
        let value = self.fence.inc_fence_value();
        self.queue
            .lock()
            .signal(self.fence.get_raw(), value)
            .unwrap();
        value
    }

    fn is_fence_complete(&self, value: u64) -> bool {
        self.fence.get_completed_value() >= value
    }

    fn wait_for_fence(&self, value: u64) {
        if !self.is_fence_complete(value) {
            let event_handle = dx::Event::create(false, false).unwrap();

            self.fence.set_event_on_completion(value, event_handle);
            event_handle.wait(u32::MAX);

            event_handle.close().unwrap();
        }
    }

    fn flush(&self) {
        self.wait_for_fence(self.signal());
    }

    pub fn execute(&self) -> u64 {
        let threads = self.pending_list.lock().drain(..).collect::<Vec<_>>();

        let lists = self.temp_buffer.lock().drain(..).collect::<Vec<_>>();

        self.queue.lock().execute_command_lists(&lists);
        let fence_value = self.signal();

        let allocators = threads.into_iter().map(|mut thread| {
            thread.allocator.inc_fence_value();
            thread.allocator
        });
        self.cmd_allocators.lock().extend(allocators);

        let lists = lists
            .into_iter()
            .map(|list| unsafe { list.unwrap_unchecked() });
        self.cmd_list.lock().extend(lists);

        fence_value
    }
}

impl<F: Fence> CommandQueue<Graphics, F> {
    pub fn graphics(device: dx::Device, fence: F) -> Self {
        Self::inner_new(device, fence, &dx::CommandQueueDesc::direct())
    }

    pub fn get_worker_thread(&self, pso: Option<&dx::PipelineState>) -> WorkerThread<Graphics> {
        let allocator = if let Some(allocator) =
            self.cmd_allocators.lock().pop_front().and_then(|mut a| {
                if self.is_fence_complete(a.fence_value()) {
                    Some(a)
                } else {
                    None
                }
            }) {
            allocator
        } else {
            CommandAllocator::inner_new(&self.device, dx::CommandListType::Direct)
        };

        let list = if let Some(list) = self.cmd_list.lock().pop() {
            list
        } else {
            self.device
                .create_command_list(0, dx::CommandListType::Direct, &allocator.raw, pso)
                .unwrap()
        };

        WorkerThread { allocator, list }
    }
}

impl<F: Fence> CommandQueue<Compute, F> {
    pub fn compute(device: dx::Device, fence: F) -> Self {
        Self::inner_new(device, fence, &dx::CommandQueueDesc::compute())
    }

    pub fn get_worker_thread(&self, pso: Option<&dx::PipelineState>) -> WorkerThread<Compute> {
        let allocator = if let Some(allocator) =
            self.cmd_allocators.lock().pop_front().and_then(|mut a| {
                if self.is_fence_complete(a.fence_value()) {
                    Some(a)
                } else {
                    None
                }
            }) {
            allocator
        } else {
            CommandAllocator::inner_new(&self.device, dx::CommandListType::Compute)
        };

        let list = if let Some(list) = self.cmd_list.lock().pop() {
            list
        } else {
            self.device
                .create_command_list(0, dx::CommandListType::Compute, &allocator.raw, pso)
                .unwrap()
        };

        WorkerThread { allocator, list }
    }
}

impl<F: Fence> CommandQueue<Transfer, F> {
    pub fn transfer(device: dx::Device, fence: F) -> Self {
        Self::inner_new(device, fence, &dx::CommandQueueDesc::copy())
    }

    pub fn get_worker_thread(&self, pso: Option<&dx::PipelineState>) -> WorkerThread<Transfer> {
        let allocator = if let Some(allocator) =
            self.cmd_allocators.lock().pop_front().and_then(|mut a| {
                if self.is_fence_complete(a.fence_value()) {
                    Some(a)
                } else {
                    None
                }
            }) {
            allocator
        } else {
            CommandAllocator::inner_new(&self.device, dx::CommandListType::Copy)
        };

        let list = if let Some(list) = self.cmd_list.lock().pop() {
            list
        } else {
            self.device
                .create_command_list(0, dx::CommandListType::Copy, &allocator.raw, pso)
                .unwrap()
        };

        WorkerThread { allocator, list }
    }
}

#[cfg(test)]
mod tests {
    use crate::fence::LocalFence;

    use super::{CommandQueue, Compute, Graphics, Transfer};

    const fn is_send_sync<T: Send + Sync>() {}

    const _: () = is_send_sync::<CommandQueue<Graphics, LocalFence>>();
    const _: () = is_send_sync::<CommandQueue<Compute, LocalFence>>();
    const _: () = is_send_sync::<CommandQueue<Transfer, LocalFence>>();
}
