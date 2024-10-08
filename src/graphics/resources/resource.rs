use atomig::Atom;
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
        state: ResourceStates,
        subresource: usize,
    ) -> Option<dx::ResourceBarrier<'_>>;
    fn get_desc(&self) -> Self::Desc;

    fn from_desc(
        device: &Device,
        desc: Self::Desc,
        access: Self::Access,
        init_state: ResourceStates,
    ) -> Self;

    fn from_raw_placed(
        heap: &MemoryHeap,
        raw: dx::Resource,
        desc: Self::Desc,
        access: Self::Access,
        state: ResourceStates,
        allocation: Allocation,
    ) -> Self;
}

pub(in super::super) trait ResourceDesc: Into<dx::ResourceDesc> + Clone {}

pub trait BufferDesc: ResourceDesc {}
pub trait TextureDesc: ResourceDesc {
    fn clear_color(&self) -> Option<&dx::ClearValue>;
    fn with_layout(self, layout: dx::TextureLayout) -> Self;
}

pub trait ShareableTextureDesc: TextureDesc {
    fn flags(&self) -> dx::ResourceFlags;
    fn with_flags(self, flags: dx::ResourceFlags) -> Self;
}

pub trait ShareableBufferDesc: BufferDesc {
    fn flags(&self) -> dx::ResourceFlags;
    fn with_flags(self, flags: dx::ResourceFlags) -> Self;
}

pub trait Buffer: Resource<Desc: BufferDesc> {}
pub trait Texture: Resource<Desc: TextureDesc> {}

pub trait ShareableBuffer: Buffer<Desc: ShareableBufferDesc> {}
pub trait ShareableTexture: Texture<Desc: ShareableTextureDesc> {}

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

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct ResourceStates: i32 {
        const Common = dx::ResourceStates::Common.bits();
        const VertexAndConstantBuffer = dx::ResourceStates::VertexAndConstantBuffer.bits();
        const IndexBuffer =  dx::ResourceStates::IndexBuffer.bits();
        const RenderTarget =  dx::ResourceStates::RenderTarget.bits();
        const UnorderedAccess = dx::ResourceStates::UnorderedAccess.bits();
        const DepthWrite =  dx::ResourceStates::DepthWrite.bits();
        const DepthRead = dx::ResourceStates::DepthRead.bits();
        const NonPixelShaderResource = dx::ResourceStates::NonPixelShaderResource.bits();
        const PixelShaderResource = dx::ResourceStates::PixelShaderResource.bits();
        const CopyDst = dx::ResourceStates::CopyDest.bits();
        const CopySrc = dx::ResourceStates::CopySource.bits();
        const GenericRead = dx::ResourceStates::GenericRead.bits();
        const AllShaderResource = dx::ResourceStates::AllShaderResource.bits();
        const Present = dx::ResourceStates::Present.bits();
    }
}

impl From<ResourceStates> for dx::ResourceStates {
    fn from(value: ResourceStates) -> Self {
        dx::ResourceStates::from_bits(value.bits()).unwrap()
    }
}

impl Atom for ResourceStates {
    type Repr = i32;

    fn pack(self) -> Self::Repr {
        self.bits()
    }

    fn unpack(src: Self::Repr) -> Self {
        ResourceStates::from_bits(src).unwrap()
    }
}
