use std::num::NonZero;

use oxidx::dx::{self, IFactory4, ISwapchain1, ISwapchain3, OUTPUT_NONE};

use super::{
    command_queue::{CommandQueue, Graphics},
    descriptor_heap::{DescriptorHeap, ResourceDescriptor, RtvHeapView},
    device::Device,
    fence::{Fence, LocalFence},
};

#[derive(Debug)]
pub struct Swapchain {
    images: Vec<dx::Resource>,
    handles: Vec<ResourceDescriptor<RtvHeapView>>,
    fence_values: Vec<u64>,
    queue: CommandQueue<Graphics, LocalFence>,
    device: Device,
    current_back_buffer: usize,
    raw: dx::Swapchain3,
}

impl Swapchain {
    pub(super) fn inner_new(
        device: Device,
        queue: CommandQueue<Graphics, LocalFence>,
        heap: &mut DescriptorHeap<RtvHeapView>,
        hwnd: NonZero<isize>,
        desc: dx::SwapchainDesc1,
        count: usize,
    ) -> Self {
        let raw = device
            .factory
            .create_swapchain_for_hwnd(&*queue.raw.lock(), hwnd, &desc, None, OUTPUT_NONE)
            .unwrap();

        let images: Vec<dx::Resource> = (0..count).map(|i| raw.get_buffer(i).unwrap()).collect();
        let fence_values = vec![0; count];
        let handles = images.iter().map(|i| heap.push(i, None)).collect();

        Self {
            images,
            handles,
            fence_values,
            queue,
            device,
            current_back_buffer: 0,
            raw: raw.try_into().unwrap(),
        }
    }
}

impl Swapchain {
    pub fn get_texture(&self) -> ResourceDescriptor<RtvHeapView> {
        self.handles[self.current_back_buffer]
    }

    pub fn present(&mut self) {
        self.raw.present(0, dx::PresentFlags::empty()).unwrap();
        self.fence_values[self.current_back_buffer] = self.queue.fence.get_current_value();
        self.current_back_buffer = self.raw.get_current_back_buffer_index() as usize;
        self.queue
            .wait_on_cpu(self.fence_values[self.current_back_buffer]);
    }
}
