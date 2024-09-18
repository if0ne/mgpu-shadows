#![allow(private_bounds)]
#![allow(private_interfaces)]

use std::{ops::Deref, sync::Arc};

use oxidx::dx::{self, IAdapter3, IDevice};

use super::{
    command_allocator::CommandAllocator,
    command_queue::{CommandQueue, Compute, Graphics, Transfer, WorkerType},
    descriptor_heap::{CbvSrvUavHeapView, DescriptorHeap, DsvHeapView, RtvHeapView},
    fence::{Fence, LocalFence, SharedFence},
    heap::SharedHeap,
    resource::SharedResource,
};

#[derive(Clone, Debug)]
pub struct Device(Arc<DeviceInner>);

impl Device {
    pub fn new(adapter: dx::Adapter3) -> Self {
        let name = adapter.get_desc1().unwrap().description().to_string();

        let raw: dx::Device = dx::create_device(Some(&adapter), dx::FeatureLevel::Level11).unwrap();

        let mut feature = dx::features::OptionsFeature::default();
        raw.check_feature_support(&mut feature).unwrap();

        Self(Arc::new(DeviceInner {
            name,
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

    pub fn create_fence(&self) -> LocalFence {
        LocalFence::inner_new(self)
    }

    pub fn create_shared_fence(&self) -> SharedFence {
        SharedFence::inner_new(self.clone())
    }

    pub fn create_shared_heap(&self, size: usize) -> SharedHeap {
        SharedHeap::inner_new(self.clone(), size)
    }

    pub fn create_shared_resource(
        &self,
        heap: &SharedHeap,
        offset: usize,
        desc: &dx::ResourceDesc,
    ) -> SharedResource {
        SharedResource::inner_new(heap, offset, desc)
    }

    pub fn is_cross_adapter_texture_supported(&self) -> bool {
        self.is_cross_adapter_texture_supported
    }
}
