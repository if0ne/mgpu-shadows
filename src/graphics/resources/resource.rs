use oxidx::dx;

use crate::graphics::{
    heaps::{Allocation, MemoryHeap},
    views::ViewAllocator,
    ResourceStates, SubresourceIndex,
};

use super::super::device::Device;

pub trait Resource {
    type Desc: ResourceDesc;
    type Access: Clone;

    fn get_raw(&self) -> &dx::Resource;
    fn get_desc(&self) -> Self::Desc;

    fn from_desc(
        device: &Device,
        desc: Self::Desc,
        access: Self::Access,
        init_state: ResourceStates,
    ) -> Self;

    fn from_raw_placed(
        heap: &MemoryHeap,
        desc: Self::Desc,
        access: Self::Access,
        state: ResourceStates,
        allocation: Allocation,
    ) -> Self;
}

pub trait ResourceDesc: Into<dx::ResourceDesc> + Clone {}

pub trait BufferResourceDesc: ResourceDesc {}
pub trait ImageResourceDesc: ResourceDesc {
    fn clear_color(&self) -> Option<dx::ClearValue>;
    fn with_layout(self, layout: dx::TextureLayout) -> Self;
}

pub trait ShareableImageDesc: ImageResourceDesc {
    fn flags(&self) -> dx::ResourceFlags;
    fn with_flags(self, flags: dx::ResourceFlags) -> Self;
}

pub trait ShareableBufferDesc: BufferResourceDesc {
    fn flags(&self) -> dx::ResourceFlags;
    fn with_flags(self, flags: dx::ResourceFlags) -> Self;
}

pub trait BufferResource: Resource<Desc: BufferResourceDesc> {
    fn get_barrier(&self, state: ResourceStates) -> Option<dx::ResourceBarrier<'_>>;
}
pub trait ImageResource: Resource<Desc: ImageResourceDesc> {
    fn get_barrier(
        &self,
        state: ResourceStates,
        subresource: Option<SubresourceIndex>,
    ) -> Option<dx::ResourceBarrier<'_>>;
}

pub trait ShareableBuffer: BufferResource<Desc: ShareableBufferDesc> {}
pub trait ShareableImage: ImageResource<Desc: ShareableImageDesc> {}

#[derive(Clone, Debug)]
pub enum GpuAccess {
    Address,
    View(ViewAllocator),
}

impl PartialEq for GpuAccess {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl Eq for GpuAccess {}

#[derive(Clone, Debug)]
pub struct ViewAccess(pub ViewAllocator);

impl From<ViewAllocator> for ViewAccess {
    fn from(value: ViewAllocator) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug)]
pub struct NoGpuAccess;
