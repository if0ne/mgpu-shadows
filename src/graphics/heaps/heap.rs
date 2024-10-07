use oxidx::dx::{self, IDevice, ResourceStates};
use std::{ops::Deref, sync::Arc};

use crate::graphics::{device::Device, resources::Resource};

#[derive(Debug)]
pub struct Allocation {
    pub(in super::super) heap: MemoryHeap,
    pub(in super::super) offset: usize,
    pub(in super::super) size: usize,
}

#[derive(Clone, Debug)]
pub struct MemoryHeap(Arc<MemoryHeapInner>);

impl MemoryHeap {
    pub(in super::super) fn inner_new(device: Device, size: usize, mtype: MemoryHeapType) -> Self {
        let desc = dx::HeapDesc::new(
            size,
            dx::HeapProperties::new(
                mtype.into(),
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

    pub(in super::super) fn create_placed_resource<R: Resource>(
        &self,
        desc: R::Desc,
        offset: usize,
        access: R::Access,
        initial_state: ResourceStates,
        optimized_clear_value: Option<&dx::ClearValue>,
    ) -> R {
        let raw_desc = desc.into();

        let resource: dx::Resource = self
            .device
            .raw
            .create_placed_resource(
                &self.heap,
                offset,
                &raw_desc,
                initial_state,
                optimized_clear_value,
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

#[derive(Debug)]
pub struct MemoryHeapInner {
    pub(in super::super) device: Device,
    pub(in super::super) heap: dx::Heap,
    pub(in super::super) size: usize,
    pub(in super::super) mtype: MemoryHeapType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryHeapType {
    Gpu,
    Cpu,
    Readback,
    Shared,
}

impl MemoryHeapType {
    fn flags(&self) -> dx::HeapFlags {
        match self {
            MemoryHeapType::Shared => dx::HeapFlags::Shared | dx::HeapFlags::SharedCrossAdapter,
            _ => dx::HeapFlags::empty(),
        }
    }
}

impl Into<dx::HeapType> for MemoryHeapType {
    fn into(self) -> dx::HeapType {
        match self {
            MemoryHeapType::Gpu => dx::HeapType::Default,
            MemoryHeapType::Cpu => dx::HeapType::Upload,
            MemoryHeapType::Readback => dx::HeapType::Readback,
            MemoryHeapType::Shared => dx::HeapType::Default,
        }
    }
}
