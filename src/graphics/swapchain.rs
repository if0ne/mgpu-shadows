use std::num::NonZero;

use oxidx::dx::{self, IDevice, IFactory4, ISwapchain1, ISwapchain3, OUTPUT_NONE};

use super::{
    command_queue::{CommandQueue, Graphics},
    descriptor_heap::{DescriptorHeap, DsvHeapView, ResourceDescriptor, RtvHeapView},
    device::Device,
    fence::{Fence, LocalFence},
};

#[derive(Debug)]
pub struct Swapchain {
    images: Vec<dx::Resource>,
    handles: Vec<ResourceDescriptor<RtvHeapView>>,
    depth: dx::Resource,
    depth_handle: ResourceDescriptor<DsvHeapView>,
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
        rtv_heap: &mut DescriptorHeap<RtvHeapView>,
        dsv_heap: &mut DescriptorHeap<DsvHeapView>,
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
        let handles = images.iter().map(|i| rtv_heap.push(i, None)).collect();

        let depth: dx::Resource = device
            .raw
            .create_committed_resource(
                &dx::HeapProperties::default(),
                dx::HeapFlags::empty(),
                &dx::ResourceDesc::texture_2d(800, 600)
                    .with_format(dx::Format::D24UnormS8Uint)
                    .with_layout(dx::TextureLayout::Unknown)
                    .with_mip_levels(1)
                    .with_flags(dx::ResourceFlags::AllowDepthStencil),
                dx::ResourceStates::Common,
                Some(&dx::ClearValue::depth(dx::Format::D24UnormS8Uint, 1.0, 0)),
            )
            .unwrap();

        let depth_handle = dsv_heap.push(&depth, None);

        Self {
            images,
            handles,
            fence_values,
            queue,
            device,
            current_back_buffer: 0,
            raw: raw.try_into().unwrap(),
            depth,
            depth_handle,
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
