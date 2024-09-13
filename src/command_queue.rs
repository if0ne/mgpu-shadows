#![allow(private_bounds)]

use std::{marker::PhantomData, ops::Deref, sync::Arc};

use oxidx::dx::{self, ICommandQueue, IDevice};
use parking_lot::Mutex;

use crate::worker_thread::WorkerThread;

pub(super) trait WorkerType {}

pub(super) struct Graphics;
impl WorkerType for Graphics {}

pub(super) struct Compute;
impl WorkerType for Compute {}

pub(super) struct Copy;
impl WorkerType for Copy {}

#[derive(Clone)]
pub struct CommandQueue<T: WorkerType>(Arc<CommandQueueInner<T>>);

impl<T: WorkerType> Deref for CommandQueue<T> {
    type Target = CommandQueueInner<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct CommandQueueInner<T: WorkerType> {
    queue: dx::CommandQueue,
    pending_list: Mutex<Vec<Option<dx::GraphicsCommandList>>>,
    _marker: PhantomData<T>,
}

impl<T: WorkerType> CommandQueue<T> {
    fn inner_new(device: &dx::Device, desc: &dx::CommandQueueDesc) -> Self {
        let queue = device.create_command_queue(desc).unwrap();

        Self(Arc::new(CommandQueueInner {
            queue,
            pending_list: Default::default(),
            _marker: PhantomData,
        }))
    }
}

impl<T: WorkerType> CommandQueueInner<T> {
    pub fn push_fiber(&self, fiber: &WorkerThread<T>) {
        self.pending_list.lock().push(Some(fiber.list.clone()));
    }

    pub fn execute(&self, fence: &dx::Fence, signal_val: u64) {
        let lists = std::mem::take(&mut *self.pending_list.lock());

        self.queue.execute_command_lists(&lists);
        self.queue.signal(fence, signal_val).unwrap();
    }
}

impl CommandQueue<Graphics> {
    pub fn graphics(device: &dx::Device) -> Self {
        Self::inner_new(device, &dx::CommandQueueDesc::direct())
    }
}

impl CommandQueue<Compute> {
    pub fn compute(device: &dx::Device) -> Self {
        Self::inner_new(device, &dx::CommandQueueDesc::compute())
    }
}

impl CommandQueue<Copy> {
    pub fn compute(device: &dx::Device) -> Self {
        Self::inner_new(device, &dx::CommandQueueDesc::copy())
    }
}
