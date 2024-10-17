use std::{cell::Cell, num::NonZero};

use oxidx::dx::{self, IFactory4, ISwapchain1, ISwapchain3, OUTPUT_NONE};

use super::{
    commands::{CommandQueue, Direct},
    device::Device,
    resources::{Image, ImageDesc},
    types::{PresentMode, SwapchainDesc, TextureUsage},
    views::{DsvView, GpuView, RtvView, SrvView, ViewAllocator},
    ResourceStates,
};

#[derive(Debug)]
pub struct Swapchain {
    raw: dx::Swapchain3,
    device: Device,
    queue: CommandQueue<Direct>,
    view_allocator: ViewAllocator,

    images: Vec<SwapchainImage>,
    depth: Image,
    desc: SwapchainDesc,

    current_back_buffer: usize,
}

impl Swapchain {
    pub(crate) fn inner_new(
        device: Device,
        queue: CommandQueue<Direct>,
        access: ViewAllocator,
        hwnd: NonZero<isize>,
        desc: SwapchainDesc,
    ) -> Self {
        let raw = device
            .factory
            .create_swapchain_for_hwnd(&*queue.raw.lock(), hwnd, &desc.as_raw(), None, OUTPUT_NONE)
            .unwrap();

        let images = (0..desc.buffer_count)
            .map(|i| {
                let raw = raw.get_buffer(i).unwrap();
                SwapchainImage {
                    rtv: access.push_rtv(&raw, None),
                    raw: Some(raw),
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
            access.clone().into(),
            ResourceStates::DepthWrite,
        );

        Self {
            raw: raw.try_into().unwrap(),
            device,
            queue,
            view_allocator: access,
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
        let image = &self.images[index];
        if let Some(srv) = image.srv.get() {
            srv
        } else {
            let handle = self
                .view_allocator
                .push_srv(image.raw.as_ref().unwrap(), None);
            image.srv.set(Some(handle));
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

    pub fn resize(&mut self, width: u32, height: u32) {
        self.images
            .iter_mut()
            .for_each(|image| image.invalidate(&self.view_allocator));

        self.desc.width = width;
        self.desc.height = height;
        self.raw
            .resize_buffers(
                self.desc.buffer_count,
                width,
                height,
                self.desc.format,
                dx::SwapchainFlags::AllowTearing | dx::SwapchainFlags::FrameLatencyWaitableObject,
            )
            .unwrap();

        self.images.iter_mut().enumerate().for_each(|(i, image)| {
            let raw = self.raw.get_buffer(i).unwrap();
            image.set_new(raw, &self.view_allocator);
        });

        self.depth = self.device.create_commited_resource(
            ImageDesc::new(width, height, dx::Format::D24UnormS8Uint).with_usage(
                TextureUsage::DepthTarget {
                    color: Some((1.0, 0)),
                    srv: true,
                },
            ),
            self.view_allocator.clone().into(),
            ResourceStates::DepthWrite,
        );
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        self.images
            .iter_mut()
            .for_each(|i| i.invalidate(&self.view_allocator));
    }
}

#[derive(Debug)]
pub struct SwapchainImage {
    raw: Option<dx::Resource>,
    rtv: GpuView<RtvView>,
    srv: Cell<Option<GpuView<SrvView>>>,
    last_access: u64,
}

impl SwapchainImage {
    fn invalidate(&mut self, allocator: &ViewAllocator) {
        if self.raw.is_none() {
            return;
        }

        self.raw = None;
        allocator.remove_rtv(self.rtv);

        if let Some(srv) = self.srv.get_mut().take() {
            allocator.remove_srv(srv);
        }
    }

    fn set_new(&mut self, raw: dx::Resource, allocator: &ViewAllocator) {
        self.rtv = allocator.push_rtv(&raw, None);
        self.raw = Some(raw);
    }
}
