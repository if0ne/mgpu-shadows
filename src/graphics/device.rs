#![allow(private_bounds)]
#![allow(private_interfaces)]

use std::{num::NonZero, ops::Deref, sync::Arc};

use oxidx::dx::{self, IAdapter3, IDevice};

use super::{
    command_allocator::CommandAllocator,
    command_queue::{CommandQueue, Compute, Graphics, Transfer, WorkerType},
    descriptor_heap::{
        CbvSrvUavHeapView, DescriptorAllocator, DescriptorHeap, DsvHeapView, RtvHeapView,
    },
    fence::{Fence, LocalFence, SharedFence},
    heaps::{MemoryHeap, MemoryHeapType},
    resources::{
        Buffer, Resource, ResourceStates, ShareableBuffer, ShareableTexture, SharedResource,
        Texture,
    },
    swapchain::Swapchain,
};

#[derive(Clone, Debug)]
pub struct Device(Arc<DeviceInner>);

impl Device {
    pub fn new(factory: dx::Factory4, adapter: dx::Adapter3) -> Self {
        let name = adapter.get_desc1().unwrap().description().to_string();

        let raw: dx::Device = dx::create_device(Some(&adapter), dx::FeatureLevel::Level11).unwrap();

        let mut feature = dx::features::OptionsFeature::default();
        raw.check_feature_support(&mut feature).unwrap();

        Self(Arc::new(DeviceInner {
            name,
            factory,
            adapter,
            raw,
            is_cross_adapter_texture_supported: feature.cross_adapter_row_major_texture_supported(),
        }))
    }
}

impl Deref for Device {
    type Target = DeviceInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct DeviceInner {
    name: String,
    pub(super) factory: dx::Factory4,
    adapter: dx::Adapter3,
    pub(super) raw: dx::Device,

    is_cross_adapter_texture_supported: bool,
}

impl Device {
    pub(super) fn create_command_allocator<T: WorkerType>(&self) -> CommandAllocator<T> {
        CommandAllocator::inner_new(&self.raw, T::RAW_TYPE)
    }
}

impl Device {
    pub fn create_graphics_command_queue<F: Fence>(&self, fence: F) -> CommandQueue<Graphics, F> {
        CommandQueue::inner_new(self.clone(), fence, &dx::CommandQueueDesc::direct())
    }

    pub fn create_compute_command_queue<F: Fence>(&self, fence: F) -> CommandQueue<Compute, F> {
        CommandQueue::inner_new(self.clone(), fence, &dx::CommandQueueDesc::direct())
    }

    pub fn create_transfer_command_queue<F: Fence>(&self, fence: F) -> CommandQueue<Transfer, F> {
        CommandQueue::inner_new(self.clone(), fence, &dx::CommandQueueDesc::direct())
    }

    pub fn create_rtv_descriptor_heap(&self, capacity: usize) -> DescriptorHeap<RtvHeapView> {
        DescriptorHeap::inner_new(self.clone(), capacity)
    }

    pub fn create_dsv_descriptor_heap(&self, capacity: usize) -> DescriptorHeap<DsvHeapView> {
        DescriptorHeap::inner_new(self.clone(), capacity)
    }

    pub fn create_cbv_srv_uav_descriptor_heap(
        &self,
        capacity: usize,
    ) -> DescriptorHeap<CbvSrvUavHeapView> {
        DescriptorHeap::inner_new(self.clone(), capacity)
    }

    pub fn create_descriptor_allocator(
        &self,
        rtv_size: usize,
        dsv_size: usize,
        cbv_srv_uav_size: usize,
        sampler_size: usize,
    ) -> DescriptorAllocator {
        DescriptorAllocator::inner_new(self, rtv_size, dsv_size, cbv_srv_uav_size, sampler_size)
    }

    pub fn create_fence(&self) -> LocalFence {
        LocalFence::inner_new(self)
    }

    pub fn create_shared_fence(&self) -> SharedFence {
        SharedFence::inner_new(self.clone())
    }

    pub fn create_heap(&self, size: usize, mtype: MemoryHeapType) -> MemoryHeap {
        MemoryHeap::inner_new(self.clone(), size, mtype)
    }

    pub fn create_commited_resource<R: Resource>(
        &self,
        desc: R::Desc,
        access: R::Access,
        init_state: ResourceStates,
    ) -> R {
        R::from_desc(&self, desc, access, init_state)
    }

    pub fn create_placed_buffer<R: Buffer>(
        &self,
        heap: &MemoryHeap,
        desc: R::Desc,
        offset: usize,
        access: R::Access,
        initial_state: ResourceStates,
    ) -> R {
        heap.create_placed_buffer(desc, offset, access, initial_state)
    }

    pub fn create_placed_texture<R: Texture>(
        &self,
        heap: &MemoryHeap,
        desc: R::Desc,
        offset: usize,
        access: R::Access,
        initial_state: ResourceStates,
    ) -> R {
        heap.create_placed_texture(desc, offset, access, initial_state)
    }

    pub fn create_shared_buffer<R: ShareableBuffer>(
        &self,
        heap: &MemoryHeap,
        offset: usize,
        desc: R::Desc,
        access: R::Access,
        local_state: ResourceStates,
        share_state: ResourceStates,
    ) -> SharedResource<R> {
        SharedResource::inner_new_buffer(heap, offset, desc, access, local_state, share_state)
    }

    pub fn create_shared_texture<R: ShareableTexture>(
        &self,
        heap: &MemoryHeap,
        offset: usize,
        desc: R::Desc,
        access: R::Access,
        local_state: ResourceStates,
        share_state: ResourceStates,
    ) -> SharedResource<R> {
        SharedResource::inner_new_texture(heap, offset, desc, access, local_state, share_state)
    }

    pub fn create_swapchain(
        &self,
        queue: CommandQueue<Graphics, LocalFence>,
        rtv_heap: &mut DescriptorHeap<RtvHeapView>,
        dsv_heap: &mut DescriptorHeap<DsvHeapView>,
        hwnd: NonZero<isize>,
        desc: dx::SwapchainDesc1,
        count: usize,
    ) -> Swapchain {
        Swapchain::inner_new(self.clone(), queue, rtv_heap, dsv_heap, hwnd, desc, count)
    }

    pub fn is_cross_adapter_texture_supported(&self) -> bool {
        self.is_cross_adapter_texture_supported
    }
}
