use std::{collections::VecDeque, marker::PhantomData, ops::Deref, sync::Arc};

use oxidx::dx::{self, ICommandQueue, IDevice, IGraphicsCommandList, PSO_NONE};
use parking_lot::Mutex;

use crate::graphics::{device::Device, fence::Fence};

use super::{command_allocator::CommandAllocator, worker_type::WorkerType, WorkerThread};

#[derive(Clone, Debug)]
pub struct CommandQueue<T: WorkerType>(Arc<CommandQueueInner<T>>);

#[derive(Debug)]
pub struct CommandQueueInner<T: WorkerType> {
    device: Device,

    pub(crate) raw: Mutex<dx::CommandQueue>,
    pub(crate) fence: Fence,

    cmd_allocators: Mutex<VecDeque<CommandAllocator<T>>>,
    cmd_list: Mutex<Vec<dx::GraphicsCommandList>>,

    pending_list: Mutex<Vec<WorkerThread<T>>>,
    temp_buffer: Mutex<Vec<Option<dx::GraphicsCommandList>>>,

    frequency: f64,

    _marker: PhantomData<T>,
}

impl<T: WorkerType> CommandQueue<T> {
    pub(crate) fn inner_new(device: Device, fence: Fence) -> Self {
        let desc = T::queue_desc();

        let queue: dx::CommandQueue = device.raw.create_command_queue(&desc).unwrap();

        let cmd_allocators = (0..3)
            .map(|_| device.create_command_allocator())
            .collect::<VecDeque<CommandAllocator<T>>>();

        let cmd_list: Vec<dx::GraphicsCommandList> = vec![device
            .raw
            .create_command_list(0, T::RAW_TYPE, &cmd_allocators[0].raw, PSO_NONE)
            .unwrap()];

        cmd_list[0].close().unwrap();

        let frequency = 1000.0 / queue.get_timestamp_frequency().unwrap() as f64;

        Self(Arc::new(CommandQueueInner {
            device,
            raw: Mutex::new(queue),
            fence,

            cmd_allocators: Mutex::new(cmd_allocators),
            cmd_list: Mutex::new(cmd_list),

            pending_list: Default::default(),
            temp_buffer: Default::default(),

            frequency,

            _marker: PhantomData,
        }))
    }
}

impl<T: WorkerType> CommandQueue<T> {
    pub fn push_worker(&self, worker: WorkerThread<T>) {
        worker.list.close().unwrap();
        self.temp_buffer.lock().push(Some(worker.list.clone()));
        self.pending_list.lock().push(worker);
    }

    pub fn wait_on_cpu(&self, value: u64) {
        if !self.is_fence_complete(value) {
            let event_handle = dx::Event::create(false, false).unwrap();

            self.fence.set_event_on_completion(value, event_handle);
            event_handle.wait(u32::MAX);

            event_handle.close().unwrap();
        }
    }

    pub fn wait_other_queue_on_gpu<OT: WorkerType>(&self, queue: &CommandQueue<OT>) {
        self.raw
            .lock()
            .wait(queue.fence.get_raw(), queue.fence.get_current_value())
            .unwrap();
    }

    pub fn wait_fence_gpu(&self, fence: &Fence) {
        self.raw
            .lock()
            .wait(fence.get_raw(), fence.get_current_value())
            .unwrap();
    }

    pub fn execute(&self) -> u64 {
        let threads = self.pending_list.lock().drain(..).collect::<Vec<_>>();

        let lists = self.temp_buffer.lock().drain(..).collect::<Vec<_>>();

        self.raw.lock().execute_command_lists(&lists);
        let fence_value = self.signal();

        let allocators = threads.into_iter().map(|mut thread| {
            thread.allocator.fence_value += 1;
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
            self.cmd_allocators.lock().pop_front().and_then(|a| {
                if self.is_fence_complete(a.fence_value) {
                    Some(a)
                } else {
                    None
                }
            }) {
            allocator.reset();
            allocator
        } else {
            self.device.create_command_allocator()
        };

        let list = if let Some(list) = self.cmd_list.lock().pop() {
            list.reset(&allocator.raw, pso).unwrap();
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
            frequency: self.frequency,
        }
    }
}

impl<T: WorkerType> CommandQueue<T> {
    fn signal(&self) -> u64 {
        let value = self.fence.inc_value();
        self.raw.lock().signal(self.fence.get_raw(), value).unwrap();
        value
    }

    fn is_fence_complete(&self, value: u64) -> bool {
        self.fence.get_completed_value() >= value
    }
}

impl<T: WorkerType> Deref for CommandQueue<T> {
    type Target = CommandQueueInner<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use crate::graphics::commands::worker_type::{Compute, Graphics, Transfer};

    use super::CommandQueue;

    const fn is_send_sync<T: Send + Sync>() {}

    const _: () = is_send_sync::<CommandQueue<Graphics>>();
    const _: () = is_send_sync::<CommandQueue<Compute>>();
    const _: () = is_send_sync::<CommandQueue<Transfer>>();
}
