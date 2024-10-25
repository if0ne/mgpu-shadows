use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::Deref,
    sync::{Arc, OnceLock},
};

use atomig::Atomic;
use oxidx::dx::{self, IDevice};

use crate::graphics::{
    device::Device,
    heaps::{Allocation, MemoryHeap},
    views::{GpuView, SrvView, UavView},
    MemoryHeapType, ResourceStates,
};

use super::{
    buffer::BaseBuffer, BufferResource, BufferResourceDesc, CounterBuffer, CounterBufferDesc,
    Resource, ResourceDesc, ViewAccess,
};

#[derive(Clone, Debug)]
pub struct StorageBuffer<T>(Arc<StorageBufferInner<T>>);

impl<T> Deref for StorageBuffer<T> {
    type Target = StorageBufferInner<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct StorageBufferInner<T> {
    buffer: BaseBuffer,
    count: usize,

    counter_buffer: CounterBuffer,
    access: ViewAccess,
    srv: OnceLock<GpuView<SrvView>>,
    uav: OnceLock<GpuView<UavView>>,

    marker: PhantomData<T>,
}

impl<T> StorageBuffer<T> {
    pub(in super::super) fn inner_new(
        device: &Device,
        resource: dx::Resource,
        desc: StorageBufferDesc<T>,
        state: ResourceStates,
        allocation: Option<Allocation>,
        access: ViewAccess,
    ) -> Self {
        let counter_buffer = CounterBuffer::from_desc(
            device,
            CounterBufferDesc::new(1),
            access.clone(),
            ResourceStates::Common,
        );

        Self(Arc::new(StorageBufferInner {
            buffer: BaseBuffer {
                raw: resource,
                size: desc.count * size_of::<T>(),
                state: Atomic::new(state),
                flags: dx::ResourceFlags::empty(),
                allocation,
            },
            count: desc.count,
            counter_buffer,
            access,
            srv: Default::default(),
            uav: Default::default(),
            marker: PhantomData,
        }))
    }
}

impl<T> StorageBuffer<T> {
    pub fn get_srv(&self) -> GpuView<SrvView> {
        let desc = self.srv.get();
        if let Some(desc) = desc {
            return *desc;
        }

        let handle = self.access.0.push_srv(
            &self.buffer.raw,
            Some(&dx::ShaderResourceViewDesc::buffer(
                dx::Format::Unknown,
                0..self.count,
                size_of::<T>(),
                dx::BufferSrvFlags::empty(),
            )),
        );
        self.srv.set(handle).unwrap();

        handle
    }

    pub fn get_uav(&self) -> GpuView<UavView> {
        let desc = self.uav.get();
        if let Some(desc) = desc {
            return *desc;
        }

        let handle = self.access.0.push_uav(
            &self.buffer.raw,
            Some(self.counter_buffer.get_raw()),
            Some(&dx::UnorderedAccessViewDesc::buffer(
                dx::Format::Unknown,
                0..self.count,
                size_of::<T>(),
                0,
                dx::BufferUavFlags::empty(),
            )),
        );
        self.uav.set(handle).unwrap();

        handle
    }

    pub fn get_counter_buffer(&self) -> &CounterBuffer {
        &self.counter_buffer
    }
}

impl<T> Resource for StorageBuffer<T> {
    type Desc = StorageBufferDesc<T>;
    type Access = ViewAccess;

    fn get_raw(&self) -> &dx::Resource {
        &self.buffer.raw
    }

    fn get_desc(&self) -> Self::Desc {
        StorageBufferDesc {
            count: self.count,
            _marker: PhantomData,
        }
    }

    fn from_desc(
        device: &Device,
        desc: Self::Desc,
        access: Self::Access,
        init_state: ResourceStates,
    ) -> Self {
        let element_byte_size = size_of::<T>();

        let resource: dx::Resource = device
            .raw
            .create_committed_resource(
                &dx::HeapProperties::default(),
                dx::HeapFlags::empty(),
                &dx::ResourceDesc::buffer(desc.count * element_byte_size),
                init_state.as_raw(),
                None,
            )
            .unwrap();

        Self::inner_new(device, resource, desc, init_state, None, access)
    }

    fn from_raw_placed(
        heap: &MemoryHeap,
        desc: Self::Desc,
        access: Self::Access,
        state: ResourceStates,
        allocation: Allocation,
    ) -> Self {
        assert!(allocation.heap.mtype == MemoryHeapType::Gpu);

        let raw_desc = desc.clone().into();

        let raw = heap
            .device
            .raw
            .create_placed_resource(
                &heap.heap,
                allocation.offset,
                &raw_desc,
                state.as_raw(),
                None,
            )
            .unwrap();

        Self::inner_new(&heap.device, raw, desc, state, Some(allocation), access)
    }
}

impl<T> BufferResource for StorageBuffer<T> {
    fn get_barrier(&self, state: ResourceStates) -> Option<dx::ResourceBarrier<'_>> {
        let old = self
            .buffer
            .state
            .swap(state, std::sync::atomic::Ordering::Relaxed);

        if old != state {
            Some(dx::ResourceBarrier::transition(
                self.get_raw(),
                old.as_raw(),
                state.as_raw(),
                None,
            ))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct StorageBufferDesc<T> {
    count: usize,
    _marker: PhantomData<T>,
}

impl<T> Clone for StorageBufferDesc<T> {
    fn clone(&self) -> Self {
        Self {
            count: self.count,
            _marker: PhantomData,
        }
    }
}

impl<T> StorageBufferDesc<T> {
    pub fn new(size: usize) -> Self {
        Self {
            count: size,
            _marker: PhantomData,
        }
    }
}

impl<T> From<StorageBufferDesc<T>> for dx::ResourceDesc {
    fn from(val: StorageBufferDesc<T>) -> Self {
        dx::ResourceDesc::buffer(val.count * size_of::<T>())
    }
}

impl<T> ResourceDesc for StorageBufferDesc<T> {}
impl<T> BufferResourceDesc for StorageBufferDesc<T> {}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::StorageBuffer;

    const fn is_send_sync<T: Send + Sync>() {}

    const _: () = is_send_sync::<StorageBuffer<u8>>();
}
