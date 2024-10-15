use oxidx::dx::{self, IDevice};
use std::{ops::Deref, sync::Arc};

use crate::graphics::{
    device::Device,
    resources::{BufferResource, ImageResource, ImageResourceDesc},
    types::MemoryHeapType,
    ResourceStates,
};

use super::Allocation;

#[derive(Clone, Debug)]
pub struct MemoryHeap(Arc<MemoryHeapInner>);

#[derive(Debug)]
pub struct MemoryHeapInner {
    pub(crate) device: Device,
    pub(crate) heap: dx::Heap,
    pub(crate) size: usize,
    pub(crate) mtype: MemoryHeapType,
}

impl MemoryHeap {
    pub(crate) fn inner_new(device: Device, size: usize, mtype: MemoryHeapType) -> Self {
        let desc = dx::HeapDesc::new(
            size,
            dx::HeapProperties::new(
                mtype.as_raw(),
                dx::CpuPageProperty::Unknown,
                dx::MemoryPool::Unknown,
            ),
        )
        .with_alignment(dx::HeapAlignment::ResourcePlacement)
        .with_flags(mtype.flags());

        let heap = device.raw.create_heap(&desc).unwrap();

        Self(Arc::new(MemoryHeapInner {
            device,
            heap: heap,
            size: size,
            mtype,
        }))
    }

    pub(in super::super) fn create_placed_buffer<R: BufferResource>(
        &self,
        desc: R::Desc,
        offset: usize,
        access: R::Access,
        initial_state: ResourceStates,
    ) -> R {
        let raw_desc = desc.clone().into();

        let resource: dx::Resource = self
            .device
            .raw
            .create_placed_resource(&self.heap, offset, &raw_desc, initial_state.as_raw(), None)
            .unwrap();

        R::from_raw_placed(
            self,
            resource,
            desc,
            access,
            initial_state,
            Allocation {
                heap: self.clone(),
                offset: offset,
                size: self.size,
            },
        )
    }

    pub(in super::super) fn create_placed_texture<R: ImageResource>(
        &self,
        desc: R::Desc,
        offset: usize,
        access: R::Access,
        initial_state: ResourceStates,
    ) -> R {
        let raw_desc = desc.clone().into();

        let resource: dx::Resource = self
            .device
            .raw
            .create_placed_resource(
                &self.heap,
                offset,
                &raw_desc,
                initial_state.as_raw(),
                desc.clear_color().as_ref(),
            )
            .unwrap();

        R::from_raw_placed(
            self,
            resource,
            desc,
            access,
            initial_state,
            Allocation {
                heap: self.clone(),
                offset: offset,
                size: self.size,
            },
        )
    }
}

impl MemoryHeap {
    pub fn connect(&self, device: Device) -> MemoryHeap {
        assert!(self.mtype == MemoryHeapType::Shared);

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
            mtype: MemoryHeapType::Shared,
        }))
    }
}

impl Deref for MemoryHeap {
    type Target = MemoryHeapInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
