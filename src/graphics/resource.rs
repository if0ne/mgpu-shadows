use std::sync::Arc;

use oxidx::dx::{self, IDevice, IResource, ResourceStates};

use super::device::Device;
use super::heap::SharedHeap;

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
    pub fn new(owner: &SharedHeap, heap_offset: usize, desc: &dx::ResourceDesc) -> Self {
        let owner_local_resource = if owner.device().is_cross_adapter_texture_supported() {
            None
        } else {
            Some(
                owner
                    .device()
                    .raw
                    .create_committed_resource(
                        &dx::HeapProperties::default(),
                        dx::HeapFlags::empty(),
                        desc,
                        dx::ResourceStates::Common,
                        Some(&dx::ClearValue::color(desc.format(), [1.0, 1.0, 1.0, 1.0])),
                    )
                    .unwrap(),
            )
        };

        let owner_cross_resource = owner
            .device()
            .raw
            .create_placed_resource(
                &owner.heap(),
                heap_offset as u64,
                desc,
                dx::ResourceStates::CopyDest,
                Some(&dx::ClearValue::color(desc.format(), [1.0, 1.0, 1.0, 1.0])),
            )
            .unwrap();

        Self {
            inner: Arc::new(SharedResourceInner {
                owner: owner.device().clone(),
                owner_local_resource,
                owner_cross_resource,
            }),
            state: SharedResourceState::Owner,
        }
    }

    pub fn connect(&mut self, other: &SharedHeap, heap_offset: usize) -> Self {
        let desc = self.inner.owner_cross_resource.get_desc();

        let cross_resource = other
            .device()
            .raw
            .create_placed_resource(
                other.heap(),
                heap_offset as u64,
                &desc,
                dx::ResourceStates::CopyDest,
                Some(&dx::ClearValue::color(desc.format(), [1.0, 1.0, 1.0, 1.0])),
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
                    Some(&dx::ClearValue::color(desc.format(), [1.0, 1.0, 1.0, 1.0])),
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
}

struct SharedResourceInner {
    owner: Device,
    owner_local_resource: Option<dx::Resource>,
    owner_cross_resource: dx::Resource,
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
