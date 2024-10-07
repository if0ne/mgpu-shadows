use std::{fmt::Debug, marker::PhantomData, ops::Deref, sync::Arc};

use oxidx::dx::{self, IDevice, IGraphicsCommandListExt, IResource};
use parking_lot::Mutex;

use crate::graphics::{
    command_queue::WorkerType,
    device::Device,
    heaps::{Allocation, MemoryHeap, MemoryHeapType},
    worker_thread::WorkerThread,
};

use super::{
    buffer::{BaseBuffer, Buffer},
    staging_buffer::{StagingBuffer, StagingBufferDesc},
    NoGpuAccess, Resource, ResourceDesc,
};

pub trait IndexBufferType: Clone {
    type Raw: Clone + Copy + Debug;

    fn format() -> dx::Format;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct U16;
impl IndexBufferType for U16 {
    type Raw = u16;

    fn format() -> dx::Format {
        dx::Format::R16Uint
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct U32;
impl IndexBufferType for U32 {
    type Raw = u32;

    fn format() -> dx::Format {
        dx::Format::R32Uint
    }
}

#[derive(Clone, Debug)]
pub struct IndexBuffer<T: IndexBufferType>(Arc<IndexBufferInner<T>>);

impl<T: IndexBufferType> Deref for IndexBuffer<T> {
    type Target = IndexBufferInner<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct IndexBufferInner<T: IndexBufferType> {
    buffer: BaseBuffer,
    count: usize,
    view: dx::IndexBufferView,
    staging_buffer: Option<StagingBuffer<T::Raw>>,
    marker: PhantomData<T>,
}

impl<T: IndexBufferType> Buffer for IndexBuffer<T> {}

impl<T: IndexBufferType> IndexBuffer<T> {
    pub(in super::super) fn inner_new(
        device: &Device,
        resource: dx::Resource,
        desc: IndexBufferDesc<T>,
        state: dx::ResourceStates,
        allocation: Option<Allocation>,
    ) -> Self {
        let view = dx::IndexBufferView::new(
            resource.get_gpu_virtual_address(),
            desc.count * size_of::<T::Raw>(),
            T::format(),
        );

        let staging_buffer = if desc.mtype == MemoryHeapType::Cpu {
            Some(StagingBuffer::from_desc(
                device,
                StagingBufferDesc::new(desc.count),
                NoGpuAccess,
                dx::ResourceStates::CopySource,
                None,
            ))
        } else {
            None
        };

        Self(Arc::new(IndexBufferInner {
            buffer: BaseBuffer {
                raw: resource,
                size: desc.count * size_of::<T>(),
                state: Mutex::new(state),
                flags: dx::ResourceFlags::empty(),
                allocation,
            },
            count: desc.count,
            staging_buffer,
            view,
            marker: PhantomData,
        }))
    }

    pub(in super::super) fn upload_data<WT: WorkerType>(
        &self,
        worker: &WorkerThread<WT>,
        src: &[T::Raw],
    ) {
        if let Some(ref staging_buffer) = self.staging_buffer {
            let src = [dx::SubresourceData::new(src)];

            worker.list.update_subresources_fixed::<1, _, _>(
                &self.buffer.raw,
                staging_buffer.get_raw(),
                0,
                0..1,
                &src,
            );
        } else {
            // TODO: Sync?
            let mut mapped = self.buffer.raw.map(0, None).unwrap();

            let slice = unsafe { std::slice::from_raw_parts_mut(mapped.as_mut(), self.count) };
            slice.clone_from_slice(src);
        }
    }
}

impl<T: IndexBufferType> IndexBuffer<T> {
    pub fn view(&self) -> dx::IndexBufferView {
        self.view
    }

    pub fn memory_type(&self) -> MemoryHeapType {
        if self.staging_buffer.is_some() {
            MemoryHeapType::Gpu
        } else {
            MemoryHeapType::Cpu
        }
    }
}

impl<T: IndexBufferType> Resource for IndexBuffer<T> {
    type Desc = IndexBufferDesc<T>;
    type Access = NoGpuAccess;

    fn get_raw(&self) -> &dx::Resource {
        &self.buffer.raw
    }

    fn get_barrier(
        &self,
        state: dx::ResourceStates,
        _subresource: usize,
    ) -> Option<dx::ResourceBarrier<'_>> {
        let mut guard = self.buffer.state.lock();
        let old = *guard;
        *guard = state;

        if old != state {
            Some(dx::ResourceBarrier::transition(self.get_raw(), old, state))
        } else {
            None
        }
    }

    fn get_desc(&self) -> Self::Desc {
        IndexBufferDesc {
            count: self.count,
            mtype: self.memory_type(),
            _marker: PhantomData,
        }
    }

    fn from_desc(
        device: &Device,
        desc: Self::Desc,
        _access: Self::Access,
        _init_state: dx::ResourceStates,
        _clear_color: Option<&dx::ClearValue>,
    ) -> Self {
        let element_byte_size = size_of::<T>();

        let resource: dx::Resource = device
            .raw
            .create_committed_resource(
                &dx::HeapProperties::default(),
                dx::HeapFlags::empty(),
                &dx::ResourceDesc::buffer(desc.count * element_byte_size),
                dx::ResourceStates::GenericRead,
                None,
            )
            .unwrap();

        Self::inner_new(
            device,
            resource,
            desc,
            dx::ResourceStates::GenericRead,
            None,
        )
    }

    fn from_raw_placed(
        heap: &MemoryHeap,
        raw: dx::Resource,
        desc: Self::Desc,
        _access: Self::Access,
        state: dx::ResourceStates,
        allocation: Allocation,
    ) -> Self {
        assert!(
            allocation.heap.mtype == MemoryHeapType::Cpu
                || allocation.heap.mtype == MemoryHeapType::Gpu
        );

        Self::inner_new(&heap.device, raw, desc, state, Some(allocation))
    }
}

#[derive(Clone, Debug)]
pub struct IndexBufferDesc<T> {
    count: usize,
    mtype: MemoryHeapType,
    _marker: PhantomData<T>,
}

impl<T> IndexBufferDesc<T> {
    pub fn new(size: usize) -> Self {
        Self {
            count: size,
            mtype: MemoryHeapType::Gpu,
            _marker: PhantomData,
        }
    }

    pub fn move_to_cpu(mut self) -> Self {
        self.mtype = MemoryHeapType::Cpu;
        self
    }
}

impl<T> Into<dx::ResourceDesc> for IndexBufferDesc<T> {
    fn into(self) -> dx::ResourceDesc {
        dx::ResourceDesc::buffer(self.count * size_of::<T>())
    }
}

impl<T: Clone> ResourceDesc for IndexBufferDesc<T> {
    fn flags(&self) -> dx::ResourceFlags {
        dx::ResourceFlags::empty()
    }

    fn with_flags(self, _flags: dx::ResourceFlags) -> Self {
        self
    }

    fn with_layout(self, _layout: dx::TextureLayout) -> Self {
        self
    }
}
