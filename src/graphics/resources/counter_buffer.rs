use std::{
    fmt::Debug,
    ops::Deref,
    sync::{Arc, OnceLock},
};

use atomig::Atomic;
use oxidx::dx::{self, IDevice};

use crate::graphics::{
    device::Device,
    heaps::{Allocation, MemoryHeap},
    types::MemoryHeapType,
    views::{GpuView, SrvView, UavView},
    ResourceStates,
};

use super::{
    buffer::BaseBuffer, BufferResource, BufferResourceDesc, Resource, ResourceDesc, ViewAccess,
};

#[derive(Clone, Debug)]
pub struct CounterBuffer(Arc<CounterBufferInner>);

impl Deref for CounterBuffer {
    type Target = CounterBufferInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct CounterBufferInner {
    buffer: BaseBuffer,
    count: usize,

    access: ViewAccess,
    srv: OnceLock<GpuView<SrvView>>,
    uav: OnceLock<GpuView<UavView>>,
}

impl CounterBuffer {
    pub(in super::super) fn inner_new(
        resource: dx::Resource,
        desc: CounterBufferDesc,
        state: ResourceStates,
        allocation: Option<Allocation>,
        access: ViewAccess,
    ) -> Self {
        Self(Arc::new(CounterBufferInner {
            buffer: BaseBuffer {
                raw: resource,
                size: desc.count * 4,
                state: Atomic::new(state),
                flags: dx::ResourceFlags::empty(),
                allocation,
            },
            count: desc.count,
            access,
            srv: Default::default(),
            uav: Default::default(),
        }))
    }
}

impl CounterBuffer {
    pub fn get_srv(&self) -> GpuView<SrvView> {
        let desc = self.srv.get();
        if let Some(desc) = desc {
            return *desc;
        }

        let handle = self.access.0.push_srv(
            &self.buffer.raw,
            Some(&dx::ShaderResourceViewDesc::buffer(
                dx::Format::R32Typeless,
                0..self.count,
                4,
                dx::BufferSrvFlags::Raw,
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
            None,
            Some(&dx::UnorderedAccessViewDesc::buffer(
                dx::Format::R32Typeless,
                0..self.count,
                4,
                0,
                dx::BufferUavFlags::Raw,
            )),
        );
        self.uav.set(handle).unwrap();

        handle
    }
}

impl Resource for CounterBuffer {
    type Desc = CounterBufferDesc;
    type Access = ViewAccess;

    fn get_raw(&self) -> &dx::Resource {
        &self.buffer.raw
    }

    fn get_desc(&self) -> Self::Desc {
        CounterBufferDesc { count: self.count }
    }

    fn from_desc(
        device: &Device,
        desc: Self::Desc,
        access: Self::Access,
        init_state: ResourceStates,
    ) -> Self {
        let resource: dx::Resource = device
            .raw
            .create_committed_resource(
                &dx::HeapProperties::upload(),
                dx::HeapFlags::empty(),
                &dx::ResourceDesc::buffer(desc.count * 4),
                init_state.as_raw(),
                None,
            )
            .unwrap();

        Self::inner_new(resource, desc, init_state, None, access)
    }

    fn from_raw_placed(
        heap: &MemoryHeap,
        desc: Self::Desc,
        access: Self::Access,
        _state: ResourceStates,
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
                ResourceStates::GenericRead.as_raw(),
                None,
            )
            .unwrap();

        Self::inner_new(
            raw,
            desc,
            ResourceStates::GenericRead,
            Some(allocation),
            access,
        )
    }
}

impl BufferResource for CounterBuffer {
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

#[derive(Clone, Debug)]
pub struct CounterBufferDesc {
    count: usize,
}

impl CounterBufferDesc {
    pub fn new(size: usize) -> Self {
        Self { count: size }
    }
}

impl From<CounterBufferDesc> for dx::ResourceDesc {
    fn from(val: CounterBufferDesc) -> Self {
        dx::ResourceDesc::buffer(val.count * 4)
    }
}

impl ResourceDesc for CounterBufferDesc {}
impl BufferResourceDesc for CounterBufferDesc {}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::CounterBuffer;

    const fn is_send_sync<T: Send + Sync>() {}

    const _: () = is_send_sync::<CounterBuffer>();
}
