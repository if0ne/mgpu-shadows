use std::{fmt::Debug, marker::PhantomData, ops::Deref, sync::Arc};

use atomig::Atomic;
use oxidx::dx::{self, IDevice, IGraphicsCommandListExt, IResource};

use crate::graphics::{
    command_queue::WorkerType,
    device::Device,
    heaps::{Allocation, MemoryHeap, MemoryHeapType},
    worker_thread::WorkerThread,
};

use super::{
    buffer::BaseBuffer,
    staging_buffer::{StagingBuffer, StagingBufferDesc},
    Buffer, BufferDesc, NoGpuAccess, Resource, ResourceDesc, ResourceStates,
};

#[derive(Clone, Debug)]
pub struct VertexBuffer<T: Clone>(Arc<VertexBufferInner<T>>);

impl<T: Clone> Deref for VertexBuffer<T> {
    type Target = VertexBufferInner<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct VertexBufferInner<T: Clone> {
    buffer: BaseBuffer,
    count: usize,
    view: dx::VertexBufferView,
    staging_buffer: Option<StagingBuffer<T>>,
    marker: PhantomData<T>,
}

impl<T: Clone> VertexBuffer<T> {
    pub(in super::super) fn inner_new(
        device: &Device,
        resource: dx::Resource,
        desc: VertexBufferDesc<T>,
        state: ResourceStates,
        allocation: Option<Allocation>,
    ) -> Self {
        let view = dx::VertexBufferView::new(
            resource.get_gpu_virtual_address(),
            size_of::<T>(),
            desc.count * size_of::<T>(),
        );

        let staging_buffer = if desc.mtype == MemoryHeapType::Cpu {
            Some(StagingBuffer::from_desc(
                device,
                StagingBufferDesc::new(desc.count),
                NoGpuAccess,
                ResourceStates::GenericRead,
            ))
        } else {
            None
        };

        Self(Arc::new(VertexBufferInner {
            buffer: BaseBuffer {
                raw: resource,
                size: desc.count * size_of::<T>(),
                state: Atomic::new(state),
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
        src: &[T],
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

impl<T: Clone> VertexBuffer<T> {
    pub fn view(&self) -> dx::VertexBufferView {
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

impl<T: Clone> Resource for VertexBuffer<T> {
    type Desc = VertexBufferDesc<T>;
    type Access = NoGpuAccess;

    fn get_raw(&self) -> &dx::Resource {
        &self.buffer.raw
    }

    fn get_barrier(
        &self,
        state: ResourceStates,
        _subresource: usize,
    ) -> Option<dx::ResourceBarrier<'_>> {
        let old = self
            .buffer
            .state
            .swap(state, std::sync::atomic::Ordering::Relaxed);

        if old != state {
            Some(dx::ResourceBarrier::transition(
                self.get_raw(),
                old,
                state,
                None,
            ))
        } else {
            None
        }
    }

    fn get_desc(&self) -> Self::Desc {
        VertexBufferDesc {
            count: self.count,
            mtype: self.memory_type(),
            _marker: PhantomData,
        }
    }

    fn from_desc(
        device: &Device,
        desc: Self::Desc,
        _access: Self::Access,
        mut init_state: ResourceStates,
    ) -> Self {
        let element_byte_size = size_of::<T>();

        if desc.mtype == MemoryHeapType::Cpu {
            init_state = ResourceStates::GenericRead;
        }

        let resource: dx::Resource = device
            .raw
            .create_committed_resource(
                &dx::HeapProperties::default(),
                dx::HeapFlags::empty(),
                &dx::ResourceDesc::buffer(desc.count * element_byte_size),
                init_state.into(),
                None,
            )
            .unwrap();

        Self::inner_new(device, resource, desc, init_state, None)
    }

    fn from_raw_placed(
        heap: &MemoryHeap,
        raw: dx::Resource,
        desc: Self::Desc,
        _access: Self::Access,
        mut state: ResourceStates,
        allocation: Allocation,
    ) -> Self {
        assert!(
            allocation.heap.mtype == MemoryHeapType::Cpu
                || allocation.heap.mtype == MemoryHeapType::Gpu
        );

        if allocation.heap.mtype == MemoryHeapType::Cpu {
            state = ResourceStates::GenericRead;
        }

        Self::inner_new(&heap.device, raw, desc, state, Some(allocation))
    }
}

impl<T: Clone> Buffer for VertexBuffer<T> {}

#[derive(Clone, Debug)]
pub struct VertexBufferDesc<T> {
    count: usize,
    mtype: MemoryHeapType,
    _marker: PhantomData<T>,
}

impl<T> VertexBufferDesc<T> {
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

impl<T> Into<dx::ResourceDesc> for VertexBufferDesc<T> {
    fn into(self) -> dx::ResourceDesc {
        dx::ResourceDesc::buffer(self.count * size_of::<T>())
    }
}

impl<T: Clone> ResourceDesc for VertexBufferDesc<T> {}
impl<T: Clone> BufferDesc for VertexBufferDesc<T> {}
