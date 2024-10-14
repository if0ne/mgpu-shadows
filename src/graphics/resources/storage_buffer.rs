use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, Range},
    sync::{Arc, OnceLock},
};

use atomig::Atomic;
use oxidx::dx::{self, IDevice, IResource};
use parking_lot::Mutex;

use crate::graphics::{
    descriptor_heap::{GpuView, SrvView, UavView},
    device::Device,
    heaps::{Allocation, MemoryHeap, MemoryHeapType},
};

use super::{
    buffer::BaseBuffer, counter_buffer, BufferResource, BufferResourceDesc, CounterBuffer,
    CounterBufferDesc, GpuOnlyDescriptorAccess, NoGpuAccess, Resource, ResourceDesc,
    ResourceStates,
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
    access: GpuOnlyDescriptorAccess,
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
        access: GpuOnlyDescriptorAccess,
    ) -> Self {
        let mapped_data = resource.map::<T>(0, None).unwrap();

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
                self.count,
                size_of::<T>(),
                dx::BufferSrvFlags::empty(),
            )),
        );
        self.srv.set(handle);

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
                self.count,
                size_of::<T>(),
                0,
                dx::BufferUavFlags::empty(),
            )),
        );
        self.uav.set(handle);

        handle
    }

    pub fn get_counter_buffer(&self) -> &CounterBuffer {
        &self.counter_buffer
    }
}

impl<T> Resource for StorageBuffer<T> {
    type Desc = StorageBufferDesc<T>;
    type Access = GpuOnlyDescriptorAccess;

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
                init_state.into(),
                None,
            )
            .unwrap();

        Self::inner_new(device, resource, desc, init_state, None, access)
    }

    fn from_raw_placed(
        heap: &MemoryHeap,
        raw: dx::Resource,
        desc: Self::Desc,
        access: Self::Access,
        state: ResourceStates,
        allocation: Allocation,
    ) -> Self {
        assert!(allocation.heap.mtype == MemoryHeapType::Gpu);

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
                old.into(),
                state.into(),
                None,
            ))
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct StorageBufferDesc<T> {
    count: usize,
    _marker: PhantomData<T>,
}

impl<T> StagingBufferDesc<T> {
    pub fn new(size: usize) -> Self {
        Self {
            count: size,
            readback: false,
            _marker: PhantomData,
        }
    }
}

impl<T> Into<dx::ResourceDesc> for StorageBufferDesc<T> {
    fn into(self) -> dx::ResourceDesc {
        dx::ResourceDesc::buffer(self.count * size_of::<T>())
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
