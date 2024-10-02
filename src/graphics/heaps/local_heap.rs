use oxidx::dx::{self, IDevice};
use parking_lot::Mutex;
use std::{marker::PhantomData, ops::Deref, sync::Arc};

use crate::graphics::{device::WeakDevice, resources::Resource};

pub trait HeapType {
    const RAW_TYPE: dx::HeapType;
    const RAW_FLAGS: dx::HeapFlags;
}

pub struct Default;
impl HeapType for Default {
    const RAW_TYPE: dx::HeapType = dx::HeapType::Default;
    const RAW_FLAGS: dx::HeapFlags = dx::HeapFlags::empty();
}

pub struct Upload;
impl HeapType for Upload {
    const RAW_TYPE: dx::HeapType = dx::HeapType::Upload;
    const RAW_FLAGS: dx::HeapFlags = dx::HeapFlags::empty();
}

pub struct Readback;
impl HeapType for Readback {
    const RAW_TYPE: dx::HeapType = dx::HeapType::Readback;
    const RAW_FLAGS: dx::HeapFlags = dx::HeapFlags::empty();
}

pub struct Shared;
impl HeapType for Shared {
    const RAW_TYPE: dx::HeapType = dx::HeapType::Default;
    const RAW_FLAGS: dx::HeapFlags = dx::HeapFlags::Shared.union(dx::HeapFlags::SharedCrossAdapter);
}

#[derive(Debug)]
pub struct Allocation<T: HeapType> {
    pub(super) heap: LocalHeap<T>,
    pub(super) offset: usize,
    pub(super) size: usize,
}

#[derive(Clone, Debug)]
pub struct LocalHeap<T: HeapType>(Arc<LocalHeapInner<T>>);

impl<T: HeapType> LocalHeap<T> {
    pub(in super::super) fn inner_new(device: WeakDevice, size: usize) -> Self {
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

        Self(Arc::new(LocalHeapInner {
            heap: heap,
            size: size,
            free_list: Mutex::new(vec![]),
            _marker: PhantomData,
        }))
    }
}

impl<T: HeapType> Deref for LocalHeap<T> {
    type Target = LocalHeapInner<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct LocalHeapInner<T: HeapType> {
    heap: dx::Heap,
    size: usize,
    free_list: Mutex<Vec<Allocation<T>>>,
    _marker: PhantomData<T>,
}

impl<T: HeapType> LocalHeapInner<T> {
    pub(in super::super) fn create_placed_resource<R: Resource>(
        &self,
        resource: R::Desc,
    ) -> Allocation<T> {
        todo!()
    }

    pub(in super::super) fn free(&self, allocation: Allocation<T>) {
        todo!()
    }
}
