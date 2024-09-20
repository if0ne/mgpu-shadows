use oxidx::dx::{self, IDevice, IResource};

use super::super::device::Device;
use super::super::heap::SharedHeap;

pub trait Resource {
    type Desc;

    fn get_desc(&self) -> Self::Desc;
}

#[derive(Clone, Debug)]
pub struct SharedResource {
    owner: Device,
    state: SharedResourceState,

    desc: dx::ResourceDesc,
}

impl SharedResource {
    pub(in super::super) fn inner_new(
        owner: &SharedHeap,
        offset: usize,
        desc: dx::ResourceDesc,
    ) -> Self {
        let flags = if owner.device().is_cross_adapter_texture_supported() {
            dx::ResourceFlags::AllowCrossAdapter | desc.flags()
        } else {
            dx::ResourceFlags::AllowCrossAdapter
        };

        let cross_desc = desc
            .clone()
            .with_flags(flags)
            .with_layout(dx::TextureLayout::RowMajor);

        let cross = owner
            .device()
            .raw
            .create_placed_resource(
                owner.heap(),
                offset as u64,
                &cross_desc,
                dx::ResourceStates::Common,
                None,
            )
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
                    dx::ResourceStates::Common,
                    None,
                )
                .unwrap();

            Self {
                owner: owner.device().clone(),
                state: SharedResourceState::Binded { cross, local },
                desc,
            }
        }
    }

    pub fn connect(&self, other: &SharedHeap, offset: usize) -> Self {
        let cross = other
            .device()
            .raw
            .create_placed_resource(
                other.heap(),
                offset as u64,
                &self.cross_resource().get_desc(),
                dx::ResourceStates::Common,
                None,
            )
            .unwrap();

        if other.device().is_cross_adapter_texture_supported() {
            Self {
                owner: other.device().clone(),
                state: SharedResourceState::CrossAdapter { cross },
                desc: self.desc.clone(),
            }
        } else {
            let local = other
                .device()
                .raw
                .create_committed_resource(
                    &dx::HeapProperties::default(),
                    dx::HeapFlags::empty(),
                    &self.desc,
                    dx::ResourceStates::Common,
                    None,
                )
                .unwrap();

            Self {
                owner: other.device().clone(),
                state: SharedResourceState::Binded { cross, local },
                desc: self.desc.clone(),
            }
        }
    }

    pub fn local_resource(&self) -> &dx::Resource {
        match &self.state {
            SharedResourceState::CrossAdapter { cross } => cross,
            SharedResourceState::Binded { local, .. } => local,
        }
    }

    pub fn cross_resource(&self) -> &dx::Resource {
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
enum SharedResourceState {
    CrossAdapter {
        cross: dx::Resource,
    },
    Binded {
        cross: dx::Resource,
        local: dx::Resource,
    },
}
