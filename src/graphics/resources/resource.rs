use oxidx::dx::{self, IDevice, IResource};

use crate::graphics::heaps::{Allocation, HeapType, MemoryHeap, Shared};

use super::super::device::Device;

pub trait Resource {
    type Desc: Into<dx::ResourceDesc>;

    fn get_desc(&self) -> Self::Desc;
    fn from_desc<H: HeapType>(device: &Device, desc: Self::Desc) -> Self;
    fn from_raw_placed<H: HeapType>(
        raw: dx::Resource,
        desc: Self::Desc,
        allocation: Allocation<H>,
    ) -> Self;
}

#[derive(Clone, Debug)]
pub struct SharedResource<R: Resource> {
    owner: Device,
    state: SharedResourceState<R>,

    desc: dx::ResourceDesc,
}

impl<R: Resource> SharedResource<R> {
    /*pub(in super::super) fn inner_new(
        owner: &SharedHeap,
        offset: usize,
        desc: dx::ResourceDesc,
        local_state: dx::ResourceStates,
        share_state: dx::ResourceStates,
        clear_color: Option<&dx::ClearValue>,
    ) -> Self {
        let (flags, state) = if owner.device().is_cross_adapter_texture_supported() {
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

        let cross = owner
            .device()
            .raw
            .create_placed_resource(owner.heap(), offset as u64, &cross_desc, state, clear_color)
            .unwrap();

        if owner.device().is_cross_adapter_texture_supported() {
            Self {
                owner: owner.device().clone(),
                state: SharedResourceState::CrossAdapter { cross },
                desc,
            }
        } else {
            let local = owner
                .device()
                .raw
                .create_committed_resource(
                    &dx::HeapProperties::default(),
                    dx::HeapFlags::empty(),
                    &desc,
                    local_state,
                    clear_color,
                )
                .unwrap();

            Self {
                owner: owner.device().clone(),
                state: SharedResourceState::Binded { cross, local },
                desc,
            }
        }
    }*/

    pub fn connect(
        &self,
        other: &MemoryHeap<Shared>,
        offset: usize,
        local_state: dx::ResourceStates,
        share_state: dx::ResourceStates,
        clear_color: Option<&dx::ClearValue>,
    ) -> Self {
        let (flags, state) = if other.device.is_cross_adapter_texture_supported() {
            (
                dx::ResourceFlags::AllowCrossAdapter | self.desc.flags(),
                local_state,
            )
        } else {
            (dx::ResourceFlags::AllowCrossAdapter, share_state)
        };

        let cross = other.create_placed_resource(
            &self.cross_resource().get_desc().with_flags(flags),
            offset,
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
            let local = other
                .device
                .raw
                .create_committed_resource(
                    &dx::HeapProperties::default(),
                    dx::HeapFlags::empty(),
                    &self.desc,
                    local_state,
                    clear_color,
                )
                .unwrap();

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

    pub fn get_desc(&self) -> &dx::ResourceDesc {
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
