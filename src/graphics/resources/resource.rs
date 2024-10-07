use oxidx::dx;

use crate::graphics::{
    descriptor_heap::DescriptorAllocator,
    heaps::{Allocation, MemoryHeap, MemoryHeapType},
};

use super::super::device::Device;

pub trait Resource {
    type Desc: ResourceDesc;
    type Access: Clone;

    fn get_raw(&self) -> &dx::Resource;
    fn set_current_state(&self, state: dx::ResourceStates) -> dx::ResourceStates;
    fn get_current_state(&self) -> dx::ResourceStates;
    fn get_desc(&self) -> Self::Desc;

    fn from_desc(
        device: &Device,
        desc: Self::Desc,
        access: Self::Access,
        init_state: dx::ResourceStates,
        clear_color: Option<&dx::ClearValue>,
    ) -> Self;

    fn from_raw_placed(
        raw: dx::Resource,
        desc: Self::Desc,
        access: Self::Access,
        state: dx::ResourceStates,
        allocation: Allocation,
    ) -> Self;
}

pub trait ResourceDesc: Into<dx::ResourceDesc> + Clone {
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
pub struct GpuOnlyDescriptorAccess(DescriptorAllocator);

#[derive(Clone, Debug)]
pub struct NoGpuAccess;

#[derive(Clone, Debug)]
pub struct SharedResource<R: Resource> {
    owner: Device,
    state: SharedResourceState<R>,

    desc: R::Desc,
}

impl<R: Resource> SharedResource<R> {
    pub(in super::super) fn inner_new(
        owner: &MemoryHeap,
        offset: usize,
        desc: R::Desc,
        access: R::Access,
        local_state: dx::ResourceStates,
        share_state: dx::ResourceStates,
        clear_color: Option<&dx::ClearValue>,
    ) -> Self {
        assert!(owner.mtype == MemoryHeapType::Shared);

        let (flags, state) = if owner.device.is_cross_adapter_texture_supported() {
            (
                dx::ResourceFlags::AllowCrossAdapter | desc.flags(),
                local_state,
            )
        } else {
            (dx::ResourceFlags::AllowCrossAdapter, share_state)
        };

        let cross_desc = desc
            .clone()
            .with_flags(flags)
            .with_layout(dx::TextureLayout::RowMajor);

        let cross =
            owner.create_placed_resource(cross_desc, offset, access.clone(), state, clear_color);

        if owner.device.is_cross_adapter_texture_supported() {
            Self {
                owner: owner.device.clone(),
                state: SharedResourceState::CrossAdapter { cross },
                desc,
            }
        } else {
            let local = R::from_desc(
                &owner.device,
                desc.clone(),
                access,
                local_state,
                clear_color,
            );

            Self {
                owner: owner.device.clone(),
                state: SharedResourceState::Binded { cross, local },
                desc,
            }
        }
    }

    pub fn connect(
        &self,
        other: &MemoryHeap,
        offset: usize,
        access: R::Access,
        local_state: dx::ResourceStates,
        share_state: dx::ResourceStates,
        clear_color: Option<&dx::ClearValue>,
    ) -> Self {
        assert!(other.mtype == MemoryHeapType::Shared);

        let (flags, state) = if other.device.is_cross_adapter_texture_supported() {
            (
                dx::ResourceFlags::AllowCrossAdapter | self.desc.flags(),
                local_state,
            )
        } else {
            (dx::ResourceFlags::AllowCrossAdapter, share_state)
        };

        let cross = other.create_placed_resource(
            self.cross_resource().get_desc().with_flags(flags),
            offset,
            access.clone(),
            state,
            clear_color,
        );

        if other.device.is_cross_adapter_texture_supported() {
            Self {
                owner: other.device.clone(),
                state: SharedResourceState::CrossAdapter { cross },
                desc: self.desc.clone(),
            }
        } else {
            let local = R::from_desc(
                &other.device,
                self.desc.clone(),
                access,
                local_state,
                clear_color,
            );

            Self {
                owner: other.device.clone(),
                state: SharedResourceState::Binded { cross, local },
                desc: self.desc.clone(),
            }
        }
    }

    pub fn local_resource(&self) -> &R {
        match &self.state {
            SharedResourceState::CrossAdapter { cross } => cross,
            SharedResourceState::Binded { local, .. } => local,
        }
    }

    pub fn cross_resource(&self) -> &R {
        match &self.state {
            SharedResourceState::CrossAdapter { cross } => cross,
            SharedResourceState::Binded { cross, .. } => cross,
        }
    }

    pub fn get_desc(&self) -> &R::Desc {
        &self.desc
    }

    pub fn owner(&self) -> &Device {
        &self.owner
    }
}

#[derive(Clone, Debug)]
enum SharedResourceState<R: Resource> {
    CrossAdapter { cross: R },
    Binded { cross: R, local: R },
}
