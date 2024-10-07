use std::{fmt::Debug, marker::PhantomData, ops::Deref, sync::Arc};

use oxidx::dx::{self, IDevice, IResource};
use parking_lot::Mutex;

use crate::graphics::{
    device::Device,
    heaps::{Allocation, MemoryHeap, MemoryHeapType},
};

use super::{
    buffer::{BaseBuffer, Buffer},
    staging_buffer::{StagingBuffer, StagingBufferDesc},
    NoGpuAccess, Resource, ResourceDesc,
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

impl<T: Clone> Buffer for VertexBuffer<T> {}

impl<T: Clone> VertexBuffer<T> {
    pub(in super::super) fn inner_new(
        device: &Device,
        resource: dx::Resource,
        desc: VertexBufferDesc<T>,
        state: dx::ResourceStates,
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
                dx::ResourceStates::CopySource,
                None,
            ))
        } else {
            None
        };

        Self(Arc::new(VertexBufferInner {
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

    fn set_current_state(&self, state: dx::ResourceStates) -> dx::ResourceStates {
        let mut guard = self.buffer.state.lock();
        let old = *guard;

        *guard = state;

        old
    }

    fn get_current_state(&self) -> dx::ResourceStates {
        *self.buffer.state.lock()
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
                dx::ResourceStates::Common,
                None,
            )
            .unwrap();

        Self::inner_new(device, resource, desc, dx::ResourceStates::Common, None)
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

impl<T: Clone> ResourceDesc for VertexBufferDesc<T> {
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
