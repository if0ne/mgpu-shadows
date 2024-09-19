use std::sync::Arc;

use oxidx::dx::{self, IDevice, IResource};

use super::super::device::Device;
use super::super::heap::SharedHeap;

pub trait Resource {
    type Desc;

    fn get_desc(&self) -> Self::Desc;
}

#[derive(Clone)]
pub struct SharedResource {
    inner: Arc<SharedResourceInner>,
    state: SharedResourceState,
}

impl SharedResource {
    pub(in super::super) fn inner_new(owner: &SharedHeap, offset: usize, desc: &dx::ResourceDesc) -> Self {
        let owner_local_resource = owner
            .device()
            .raw
            .create_committed_resource(
                &dx::HeapProperties::default(),
                dx::HeapFlags::empty(),
                desc,
                dx::ResourceStates::Common,
                None,
            )
            .unwrap();

        let owner_cross_resource = if owner.device().is_cross_adapter_texture_supported() {
            None
        } else {
            Some(
                owner
                    .device()
                    .raw
                    .create_placed_resource(
                        owner.heap(),
                        offset as u64,
                        &dx::ResourceDesc::texture_2d(desc.width(), desc.height())
                            .with_format(desc.format())
                            .with_layout(dx::TextureLayout::RowMajor)
                            .with_mip_levels(1)
                            .with_flags(dx::ResourceFlags::AllowCrossAdapter),
                        dx::ResourceStates::Common,
                        None,
                    )
                    .unwrap(),
            )
        };

        Self {
            inner: Arc::new(SharedResourceInner {
                owner: owner.device().clone(),
                owner_local_resource,
                owner_cross_resource,
            }),
            state: SharedResourceState::Owner,
        }
    }

    pub fn connect(&self, other: &SharedHeap, offset: usize) -> Self {
        let desc = self.inner.owner_local_resource.get_desc();

        let cross_resource = other
            .device()
            .raw
            .create_placed_resource(
                other.heap(),
                offset as u64,
                &dx::ResourceDesc::texture_2d(desc.width(), desc.height())
                    .with_format(desc.format())
                    .with_layout(dx::TextureLayout::RowMajor)
                    .with_mip_levels(1)
                    .with_flags(dx::ResourceFlags::AllowCrossAdapter),
                dx::ResourceStates::Common,
                None,
            )
            .unwrap();

        if other.device().is_cross_adapter_texture_supported() {
            Self {
                inner: Arc::clone(&self.inner),
                state: SharedResourceState::CrossAdapter {
                    cross: cross_resource,
                },
            }
        } else {
            let local_resource = other
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
                inner: Arc::clone(&self.inner),
                state: SharedResourceState::Connected {
                    cross: cross_resource,
                    local: local_resource,
                },
            }
        }
    }

    pub fn local_resource(&self) -> &dx::Resource {
        match &self.state {
            SharedResourceState::Owner => &self.inner.owner_local_resource,
            SharedResourceState::CrossAdapter { cross } => cross,
            SharedResourceState::Connected { local, .. } => local,
        }
    }

    pub fn cross_resource(&self) -> &dx::Resource {
        match &self.state {
            SharedResourceState::Owner => self
                .inner
                .owner_cross_resource
                .as_ref()
                .unwrap_or(&self.inner.owner_local_resource),
            SharedResourceState::CrossAdapter { cross } => cross,
            SharedResourceState::Connected { cross, .. } => cross,
        }
    }

    pub fn get_desc(&self) -> dx::ResourceDesc {
        self.inner.owner_local_resource.get_desc()
    }

    pub fn owner(&self) -> &Device {
        &self.inner.owner
    }
}

struct SharedResourceInner {
    owner: Device,
    owner_local_resource: dx::Resource,
    owner_cross_resource: Option<dx::Resource>,
}

#[derive(Clone)]
enum SharedResourceState {
    Owner,
    CrossAdapter {
        cross: dx::Resource,
    },
    Connected {
        cross: dx::Resource,
        local: dx::Resource,
    },
}
