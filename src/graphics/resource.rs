use std::sync::Arc;

use oxidx::dx::{self, IDevice, IResource, ResourceStates};

pub trait Resource {
    type Desc;

    fn get_desc(&self) -> Self::Desc;
}

pub struct SharedResource {
    inner: Arc<SharedResourceInner>,
    state: SharedResourceState,
}

impl SharedResource {
    pub fn new(owner: dx::Device, desc: &dx::ResourceDesc) -> Self {
        let owner_resource = owner
            .create_committed_resource(
                &dx::HeapProperties::default(),
                dx::HeapFlags::Shared | dx::HeapFlags::SharedCrossAdapter,
                desc,
                ResourceStates::Common,
                None,
            )
            .unwrap();

        Self {
            inner: Arc::new(SharedResourceInner {
                owner,
                owner_resource,
            }),
            state: SharedResourceState::Owner,
        }
    }

    pub fn connect(&mut self, device: &dx::Device) -> SharedResource {
        let desc = self.inner.owner_resource.get_desc();
        let handle = self
            .inner
            .owner
            .create_shared_handle(&self.inner.owner_resource.clone().into(), None)
            .unwrap();

        handle.close().unwrap();

        todo!()
        /*Self {
            inner: Arc::clone(&self.inner),
            state: SharedResourceState::Connected { fence },
        }*/
    }
}

struct SharedResourceInner {
    owner: dx::Device,
    owner_resource: dx::Resource,
}

enum SharedResourceState {
    Owner,
    Connected { resource: dx::Resource },
}
