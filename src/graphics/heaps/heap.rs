use oxidx::dx::{self, IDevice, ResourceStates};
use std::{marker::PhantomData, ops::Deref, sync::Arc};

use crate::graphics::{device::Device, resources::Resource};

pub trait HeapType: Clone {
    const RAW_TYPE: dx::HeapType;
    const RAW_FLAGS: dx::HeapFlags;
}

#[derive(Clone, Copy, Debug)]
pub struct Default;
impl HeapType for Default {
    const RAW_TYPE: dx::HeapType = dx::HeapType::Default;
    const RAW_FLAGS: dx::HeapFlags = dx::HeapFlags::empty();
}

#[derive(Clone, Copy, Debug)]
pub struct Upload;
impl HeapType for Upload {
    const RAW_TYPE: dx::HeapType = dx::HeapType::Upload;
    const RAW_FLAGS: dx::HeapFlags = dx::HeapFlags::empty();
}

#[derive(Clone, Copy, Debug)]
pub struct Readback;
impl HeapType for Readback {
    const RAW_TYPE: dx::HeapType = dx::HeapType::Readback;
    const RAW_FLAGS: dx::HeapFlags = dx::HeapFlags::empty();
}

#[derive(Clone, Copy, Debug)]
pub struct Shared;
impl HeapType for Shared {
    const RAW_TYPE: dx::HeapType = dx::HeapType::Default;
    const RAW_FLAGS: dx::HeapFlags = dx::HeapFlags::Shared.union(dx::HeapFlags::SharedCrossAdapter);
}

#[derive(Debug)]
pub struct Allocation<T: HeapType> {
    pub(super) heap: MemoryHeap<T>,
    pub(super) offset: usize,
    pub(super) size: usize,
}

#[derive(Clone, Debug)]
pub struct MemoryHeap<T: HeapType>(Arc<MemoryHeapInner<T>>);

impl<T: HeapType> MemoryHeap<T> {
    pub(in super::super) fn inner_new(device: Device, size: usize) -> Self {
        let desc = dx::HeapDesc::new(
            size,
            dx::HeapProperties::new(
                T::RAW_TYPE,
                dx::CpuPageProperty::Unknown,
                dx::MemoryPool::Unknown,
            ),
        )
        .with_alignment(dx::HeapAlignment::ResourcePlacement)
        .with_flags(T::RAW_FLAGS);

        let heap = device.raw.create_heap(&desc).unwrap();

        Self(Arc::new(MemoryHeapInner {
            device,
            heap: heap,
            size: size,
            _marker: PhantomData,
        }))
    }

    pub(in super::super) fn create_placed_resource<R: Resource>(
        &self,
        desc: R::Desc,
        offset: usize,
        initial_state: ResourceStates,
        optimized_clear_value: Option<&dx::ClearValue>,
    ) -> R {
        let desc = desc.into();

        let resource: dx::Resource = self
            .device
            .raw
            .create_placed_resource(
                &self.heap,
                offset,
                &desc,
                initial_state,
                optimized_clear_value,
            )
            .unwrap();

        R::from_raw_placed(
            resource,
            Allocation {
                heap: self.clone(),
                offset: offset,
                size: self.size,
            },
        )
    }
}

impl MemoryHeap<Shared> {
    pub fn connect(&self, device: Device) -> MemoryHeap<Shared> {
        let handle = self
            .device
            .raw
            .create_shared_handle(&self.heap, None)
            .unwrap();
        let heap = device.raw.open_shared_handle(handle).unwrap();
        handle.close().unwrap();

        MemoryHeap(Arc::new(MemoryHeapInner {
            device,
            heap,
            size: self.size,
            _marker: PhantomData,
        }))
    }
}

impl<T: HeapType> Deref for MemoryHeap<T> {
    type Target = MemoryHeapInner<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct MemoryHeapInner<T: HeapType> {
    pub(in super::super) device: Device,
    pub(in super::super) heap: dx::Heap,
    size: usize,
    _marker: PhantomData<T>,
}
