use std::{fmt::Debug, marker::PhantomData, ops::Deref, ptr::NonNull, sync::Arc};

use atomig::Atomic;
use oxidx::dx::{self, IDevice, IResource};
use parking_lot::Mutex;

use crate::graphics::{
    device::Device,
    heaps::{Allocation, MemoryHeap, MemoryHeapType},
};

use super::{
    buffer::BaseBuffer, Buffer, BufferDesc, NoGpuAccess, Resource, ResourceDesc, ResourceStates,
};

#[derive(Clone, Debug)]
pub struct StagingBuffer<T: Clone>(Arc<StagingBufferInner<T>>);

impl<T: Clone> Deref for StagingBuffer<T> {
    type Target = StagingBufferInner<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct StagingBufferInner<T: Clone> {
    buffer: BaseBuffer,
    count: usize,
    mapped_data: Mutex<NonNull<T>>,
    marker: PhantomData<T>,
}

impl<T: Clone> StagingBuffer<T> {
    pub(in super::super) fn inner_new(
        resource: dx::Resource,
        desc: StagingBufferDesc<T>,
        state: ResourceStates,
        allocation: Option<Allocation>,
    ) -> Self {
        let mapped_data = resource.map::<T>(0, None).unwrap();

        Self(Arc::new(StagingBufferInner {
            buffer: BaseBuffer {
                raw: resource,
                size: desc.count * size_of::<T>(),
                state: Atomic::new(state),
                flags: dx::ResourceFlags::empty(),
                allocation,
            },
            count: desc.count,
            marker: PhantomData,
            mapped_data: Mutex::new(mapped_data),
        }))
    }
}

impl<T: Clone> StagingBuffer<T> {
    pub fn write_data(&self, src: &[T]) {
        assert_eq!(src.len(), self.count);

        let mut guard = self.mapped_data.lock();
        let slice = unsafe { std::slice::from_raw_parts_mut(guard.as_mut(), self.count) };

        slice.clone_from_slice(src);
    }
}

impl<T: Clone> Drop for StagingBuffer<T> {
    fn drop(&mut self) {
        self.buffer.raw.unmap(0, None);
    }
}

impl<T: Clone> Resource for StagingBuffer<T> {
    type Desc = StagingBufferDesc<T>;
    type Access = NoGpuAccess;

    fn get_raw(&self) -> &dx::Resource {
        &self.buffer.raw
    }

    fn get_barrier(
        &self,
        _state: ResourceStates,
        _subresource: usize,
    ) -> Option<dx::ResourceBarrier<'_>> {
        None
    }

    fn get_desc(&self) -> Self::Desc {
        StagingBufferDesc {
            count: self.count,
            _marker: PhantomData,
        }
    }

    fn from_desc(
        device: &Device,
        desc: Self::Desc,
        _access: Self::Access,
        init_state: ResourceStates,
    ) -> Self {
        assert_eq!(init_state, ResourceStates::GenericRead);

        let element_byte_size = size_of::<T>();

        let resource: dx::Resource = device
            .raw
            .create_committed_resource(
                &dx::HeapProperties::upload(),
                dx::HeapFlags::empty(),
                &dx::ResourceDesc::buffer(desc.count * element_byte_size),
                init_state.into(),
                None,
            )
            .unwrap();

        Self::inner_new(resource, desc, init_state, None)
    }

    fn from_raw_placed(
        _heap: &MemoryHeap,
        raw: dx::Resource,
        desc: Self::Desc,
        _access: Self::Access,
        state: ResourceStates,
        allocation: Allocation,
    ) -> Self {
        assert!(allocation.heap.mtype == MemoryHeapType::Cpu);
        assert!(state == ResourceStates::GenericRead);

        Self::inner_new(raw, desc, state, Some(allocation))
    }
}

impl<T: Clone> Buffer for StagingBuffer<T> {}

#[derive(Clone, Debug)]
pub struct StagingBufferDesc<T> {
    count: usize,
    _marker: PhantomData<T>,
}

impl<T> StagingBufferDesc<T> {
    pub fn new(size: usize) -> Self {
        Self {
            count: size,
            _marker: PhantomData,
        }
    }
}

impl<T> Into<dx::ResourceDesc> for StagingBufferDesc<T> {
    fn into(self) -> dx::ResourceDesc {
        dx::ResourceDesc::buffer(self.count * size_of::<T>())
    }
}

impl<T: Clone> ResourceDesc for StagingBufferDesc<T> {}
impl<T: Clone> BufferDesc for StagingBufferDesc<T> {}
