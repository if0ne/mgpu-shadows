use oxidx::dx;

use crate::graphics::{
    descriptor_heap::DescriptorAllocator,
    heaps::{Allocation, MemoryHeap},
};

use super::super::device::Device;

pub(in super::super) trait Resource {
    type Desc: ResourceDesc;
    type Access: Clone;

    fn get_raw(&self) -> &dx::Resource;
    fn get_barrier(
        &self,
        state: dx::ResourceStates,
        subresource: usize,
    ) -> Option<dx::ResourceBarrier<'_>>;
    fn get_desc(&self) -> Self::Desc;

    fn from_desc(
        device: &Device,
        desc: Self::Desc,
        access: Self::Access,
        init_state: dx::ResourceStates,
        clear_color: Option<&dx::ClearValue>,
    ) -> Self;

    fn from_raw_placed(
        heap: &MemoryHeap,
        raw: dx::Resource,
        desc: Self::Desc,
        access: Self::Access,
        state: dx::ResourceStates,
        allocation: Allocation,
    ) -> Self;
}

pub(in super::super) trait ResourceDesc: Into<dx::ResourceDesc> + Clone {
    fn flags(&self) -> dx::ResourceFlags;
    fn with_flags(self, flags: dx::ResourceFlags) -> Self;
    fn with_layout(self, layout: dx::TextureLayout) -> Self;
}

#[derive(Clone, Debug)]
pub enum GpuAccess {
    Address,
    Descriptor(DescriptorAllocator),
}

impl PartialEq for GpuAccess {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Eq for GpuAccess {}

#[derive(Clone, Debug)]
pub struct GpuOnlyDescriptorAccess(pub DescriptorAllocator);

#[derive(Clone, Debug)]
pub struct NoGpuAccess;

