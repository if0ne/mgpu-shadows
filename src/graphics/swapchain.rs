use std::{cell::Cell, num::NonZero};

use oxidx::dx::{self, IFactory4, ISwapchain1, ISwapchain3, OUTPUT_NONE};

use super::{
    command_queue::{CommandQueue, Graphics},
    descriptor_heap::{DescriptorAllocator, DsvView, GpuView, RtvView, SrvView},
    device::Device,
    resources::{Image, ImageDesc, ResourceStates, TextureUsage},
};

#[derive(Debug)]
pub struct Swapchain {
    raw: dx::Swapchain3,
    device: Device,
    queue: CommandQueue<Graphics>,
    descriptor_allocator: DescriptorAllocator,

    images: Vec<SwapchainImage>,
    depth: Image,
    desc: SwapchainDesc,

    current_back_buffer: usize,
}

impl Swapchain {
    pub(super) fn inner_new(
        device: Device,
        queue: CommandQueue<Graphics>,
        descriptor_allocator: DescriptorAllocator,
        hwnd: NonZero<isize>,
        desc: SwapchainDesc,
    ) -> Self {
        let raw = device
            .factory
            .create_swapchain_for_hwnd(
                &*queue.raw.lock(),
                hwnd,
                &desc.clone().into(),
                None,
                OUTPUT_NONE,
            )
            .unwrap();

        let images = (0..desc.buffer_count)
            .map(|i| {
                let raw = raw.get_buffer(i).unwrap();
                SwapchainImage {
                    rtv: descriptor_allocator.push_rtv(&raw, None),
                    raw,
                    srv: Default::default(),
                    last_access: 0,
                }
            })
            .collect();

        let depth: Image = device.create_commited_resource(
            ImageDesc::new(desc.width, desc.height, dx::Format::D24UnormS8Uint).with_usage(
                TextureUsage::DepthTarget {
                    color: Some((1.0, 0)),
                    srv: true,
                },
            ),
            descriptor_allocator.clone().into(),
            ResourceStates::DepthWrite,
        );

        Self {
            raw: raw.try_into().unwrap(),
            device,
            queue,
            descriptor_allocator,
            images,
            depth,
            desc,
            current_back_buffer: 0,
        }
    }
}

impl Swapchain {
    pub fn get_rtv(&self) -> GpuView<RtvView> {
        self.queue
            .wait_on_cpu(self.images[self.current_back_buffer].last_access);

        self.images[self.current_back_buffer].rtv
    }

    pub fn get_rendet_target_as_srv(&self, index: usize) -> GpuView<SrvView> {
        if let Some(srv) = self.images[index].srv.get() {
            srv
        } else {
            let handle = self
                .descriptor_allocator
                .push_srv(&self.images[index].raw, None);
            self.images[index].srv.set(Some(handle));
            handle
        }
    }

    pub fn get_dsv(&self) -> GpuView<DsvView> {
        self.depth.dsv(None)
    }

    pub fn get_depth_as_srv(&self) -> GpuView<SrvView> {
        self.depth.srv(None)
    }

    pub fn present(&mut self) {
        let (interval, flags) = match self.desc.present_mode {
            PresentMode::Immediate => (0, dx::PresentFlags::AllowTearing),
            PresentMode::Mailbox => (0, dx::PresentFlags::empty()),
            PresentMode::Fifo => (1, dx::PresentFlags::empty()),
        };

        self.raw.present(interval, flags).unwrap();
        self.images[self.current_back_buffer].last_access = self.queue.fence.get_current_value();
        self.current_back_buffer = self.raw.get_current_back_buffer_index() as usize;
    }
}

#[derive(Clone, Debug)]
pub struct SwapchainDesc {
    pub width: u32,
    pub height: u32,
    pub format: dx::Format,
    pub buffer_count: usize,
    pub present_mode: PresentMode,
}

#[derive(Clone, Debug)]
pub enum PresentMode {
    Immediate,
    Mailbox,
    Fifo,
}

impl From<SwapchainDesc> for dx::SwapchainDesc1 {
    fn from(value: SwapchainDesc) -> Self {
        Self::new(value.width, value.height)
            .with_format(value.format)
            .with_buffer_count(value.buffer_count)
            .with_usage(dx::FrameBufferUsage::RenderTargetOutput)
            .with_scaling(dx::Scaling::Stretch)
            .with_swap_effect(dx::SwapEffect::FlipDiscard)
            .with_flags(
                dx::SwapchainFlags::AllowTearing | dx::SwapchainFlags::FrameLatencyWaitableObject,
            )
    }
}

#[derive(Debug)]
pub struct SwapchainImage {
    raw: dx::Resource,
    rtv: GpuView<RtvView>,
    srv: Cell<Option<GpuView<SrvView>>>,
    last_access: u64,
}
