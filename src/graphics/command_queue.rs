#![allow(private_bounds)]

use std::{collections::VecDeque, marker::PhantomData, ops::Deref, sync::Arc};

use oxidx::dx::{self, ICommandQueue, IDevice, PSO_NONE};
use parking_lot::Mutex;

use super::{
    command_allocator::CommandAllocator, device::Device, fence::Fence, worker_thread::WorkerThread,
};

pub(super) trait WorkerType {
    const RAW_TYPE: dx::CommandListType;
}

pub(super) struct Graphics;
impl WorkerType for Graphics {
    const RAW_TYPE: dx::CommandListType = dx::CommandListType::Direct;
}

pub(super) struct Compute;
impl WorkerType for Compute {
    const RAW_TYPE: dx::CommandListType = dx::CommandListType::Compute;
}

pub(super) struct Transfer;
impl WorkerType for Transfer {
    const RAW_TYPE: dx::CommandListType = dx::CommandListType::Copy;
}

#[derive(Clone)]
pub struct CommandQueue<T: WorkerType, F: Fence>(Arc<CommandQueueInner<T, F>>);

impl<T: WorkerType, F: Fence> Deref for CommandQueue<T, F> {
    type Target = CommandQueueInner<T, F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct CommandQueueInner<T: WorkerType, F: Fence> {
    device: Device,

    queue: Mutex<dx::CommandQueue>,

    cmd_allocators: Mutex<VecDeque<CommandAllocator<T>>>,
    cmd_list: Mutex<Vec<dx::GraphicsCommandList>>,

    pending_list: Mutex<Vec<WorkerThread<T>>>,
    temp_buffer: Mutex<Vec<Option<dx::GraphicsCommandList>>>,
    fence: F,
    _marker: PhantomData<T>,
}

impl<T: WorkerType, F: Fence> CommandQueue<T, F> {
    pub(super) fn inner_new(device: Device, fence: F, desc: &dx::CommandQueueDesc) -> Self {
        let queue = device.raw.create_command_queue(desc).unwrap();

        let cmd_allocators = (0..3)
            .map(|_| device.create_command_allocator())
            .collect::<VecDeque<CommandAllocator<T>>>();

        let cmd_list = vec![device
            .raw
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
    fn signal(&self) -> u64 {
        let value = self.fence.inc_value();
        self.queue
            .lock()
            .signal(self.fence.get_raw(), value)
            .unwrap();
        value
    }

    fn is_fence_complete(&self, value: u64) -> bool {
        self.fence.get_completed_value() >= value
    }
}

impl<T: WorkerType, F: Fence> CommandQueueInner<T, F> {
    pub fn push_fiber(&self, fiber: WorkerThread<T>) {
        self.temp_buffer.lock().push(Some(fiber.list.clone()));
        self.pending_list.lock().push(fiber);
    }

    pub fn wait_on_cpu(&self, value: u64) {
        if !self.is_fence_complete(value) {
            let event_handle = dx::Event::create(false, false).unwrap();

            self.fence.set_event_on_completion(value, event_handle);
            event_handle.wait(u32::MAX);

            event_handle.close().unwrap();
        }
    }

    pub fn wait_other_queue_on_gpu<OT: WorkerType, OF: Fence>(&self, queue: CommandQueue<OT, OF>) {
        self.queue
            .lock()
            .wait(queue.fence.get_raw(), queue.fence.get_current_value())
            .unwrap();
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

    pub fn get_worker_thread(&self, pso: Option<&dx::PipelineState>) -> WorkerThread<T> {
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
            self.device.create_command_allocator()
        };

        let list = if let Some(list) = self.cmd_list.lock().pop() {
            list
        } else {
            self.device
                .raw
                .create_command_list(0, T::RAW_TYPE, &allocator.raw, pso)
                .unwrap()
        };

        WorkerThread {
            device: self.device.clone(),
            allocator,
            list,
        }
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::super::fence::LocalFence;

    use super::{CommandQueue, Compute, Graphics, Transfer};

    const fn is_send_sync<T: Send + Sync>() {}

    const _: () = is_send_sync::<CommandQueue<Graphics, LocalFence>>();
    const _: () = is_send_sync::<CommandQueue<Compute, LocalFence>>();
    const _: () = is_send_sync::<CommandQueue<Transfer, LocalFence>>();
}