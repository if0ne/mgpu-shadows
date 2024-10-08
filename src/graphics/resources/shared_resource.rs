use oxidx::dx;

use crate::graphics::{
    heaps::{MemoryHeap, MemoryHeapType},
    resources::{ShareableBufferDesc, ShareableTextureDesc, TextureDesc},
};

use super::{super::device::Device, Resource, ResourceStates, ShareableBuffer, ShareableTexture};

#[derive(Clone, Debug)]
pub struct SharedResource<R: Resource> {
    owner: Device,
    state: SharedResourceState<R>,

    desc: R::Desc,
}

#[derive(Clone, Debug)]
enum SharedResourceState<R: Resource> {
    CrossAdapter { cross: R },
    Binded { cross: R, local: R },
}

impl<R: Resource> SharedResource<R> {
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

impl<R: ShareableTexture> SharedResource<R> {
    pub(in super::super) fn inner_new_texture(
        owner: &MemoryHeap,
        offset: usize,
        desc: R::Desc,
        access: R::Access,
        local_state: ResourceStates,
        share_state: ResourceStates,
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

        let cross = owner.create_placed_texture(cross_desc, offset, access.clone(), state);

        if owner.device.is_cross_adapter_texture_supported() {
            Self {
                owner: owner.device.clone(),
                state: SharedResourceState::CrossAdapter { cross },
                desc,
            }
        } else {
            let local = R::from_desc(&owner.device, desc.clone(), access, local_state);

            Self {
                owner: owner.device.clone(),
                state: SharedResourceState::Binded { cross, local },
                desc,
            }
        }
    }

    pub fn connect_texture(
        &self,
        other: &MemoryHeap,
        offset: usize,
        access: R::Access,
        local_state: ResourceStates,
        share_state: ResourceStates,
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

        let cross = other.create_placed_texture(
            self.cross_resource().get_desc().with_flags(flags),
            offset,
            access.clone(),
            state,
        );

        if other.device.is_cross_adapter_texture_supported() {
            Self {
                owner: other.device.clone(),
                state: SharedResourceState::CrossAdapter { cross },
                desc: self.desc.clone(),
            }
        } else {
            let local = R::from_desc(&other.device, self.desc.clone(), access, local_state);

            Self {
                owner: other.device.clone(),
                state: SharedResourceState::Binded { cross, local },
                desc: self.desc.clone(),
            }
        }
    }
}

impl<R: ShareableBuffer> SharedResource<R> {
    pub(in super::super) fn inner_new_buffer(
        owner: &MemoryHeap,
        offset: usize,
        desc: R::Desc,
        access: R::Access,
        local_state: ResourceStates,
        share_state: ResourceStates,
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

        let cross_desc = desc.clone().with_flags(flags);

        let cross = owner.create_placed_buffer(cross_desc, offset, access.clone(), state);

        if owner.device.is_cross_adapter_texture_supported() {
            Self {
                owner: owner.device.clone(),
                state: SharedResourceState::CrossAdapter { cross },
                desc,
            }
        } else {
            let local = R::from_desc(&owner.device, desc.clone(), access, local_state);

            Self {
                owner: owner.device.clone(),
                state: SharedResourceState::Binded { cross, local },
                desc,
            }
        }
    }

    pub fn connect_buffer(
        &self,
        other: &MemoryHeap,
        offset: usize,
        access: R::Access,
        local_state: ResourceStates,
        share_state: ResourceStates,
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

        let cross = other.create_placed_buffer(
            self.cross_resource().get_desc().with_flags(flags),
            offset,
            access.clone(),
            state,
        );

        if other.device.is_cross_adapter_texture_supported() {
            Self {
                owner: other.device.clone(),
                state: SharedResourceState::CrossAdapter { cross },
                desc: self.desc.clone(),
            }
        } else {
            let local = R::from_desc(&other.device, self.desc.clone(), access, local_state);

            Self {
                owner: other.device.clone(),
                state: SharedResourceState::Binded { cross, local },
                desc: self.desc.clone(),
            }
        }
    }
}
