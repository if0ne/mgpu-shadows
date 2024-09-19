#![allow(private_bounds)]

use super::{
    command_allocator::CommandAllocator,
    command_queue::{Graphics, WorkerType},
    device::Device,
    resources::SharedResource,
};

use oxidx::dx::{self, IDevice, IGraphicsCommandList};

pub struct WorkerThread<T: WorkerType> {
    pub(super) device: Device,
    pub(super) allocator: CommandAllocator<T>,
    pub(super) list: dx::GraphicsCommandList,
}

impl<T: WorkerType> WorkerThread<T> {
    fn inner_new(
        device: Device,
        allocator: CommandAllocator<T>,
        r#type: dx::CommandListType,
    ) -> Self {
        let list = device
            .raw
            .create_command_list(0, r#type, &allocator.raw, dx::PSO_NONE)
            .unwrap();

        Self {
            device,
            list,
            allocator,
        }
    }

    pub fn close(&self) {
        self.list.close().unwrap();
    }

    pub fn pull_shared(&self, shared_resource: &SharedResource) {
        if self.device.is_cross_adapter_texture_supported() {
            return;
        }

        self.list.resource_barrier(&[
            dx::ResourceBarrier::transition(
                shared_resource.local_resource(),
                dx::ResourceStates::Common,
                dx::ResourceStates::CopyDest,
            ),
            dx::ResourceBarrier::transition(
                shared_resource.cross_resource(),
                dx::ResourceStates::Common,
                dx::ResourceStates::CopySource,
            ),
        ]);

        self.list.copy_resource(
            shared_resource.local_resource(),
            shared_resource.cross_resource(),
        );
    }

    pub fn push_shared(&self, shared_resource: &SharedResource) {
        if self.device.is_cross_adapter_texture_supported() {
            return;
        }

        self.list.resource_barrier(&[
            dx::ResourceBarrier::transition(
                shared_resource.cross_resource(),
                dx::ResourceStates::Common,
                dx::ResourceStates::CopyDest,
            ),
            dx::ResourceBarrier::transition(
                shared_resource.local_resource(),
                dx::ResourceStates::Common,
                dx::ResourceStates::CopySource,
            ),
        ]);

        self.list.copy_resource(
            shared_resource.cross_resource(),
            shared_resource.local_resource(),
        );
    }

    pub fn barrier(&self, barriers: &[dx::ResourceBarrier<'_>]) {
        self.list.resource_barrier(barriers);
    }
}

impl WorkerThread<Graphics> {
    pub fn clear_rt(&self, handle: dx::CpuDescriptorHandle, color: [f32; 4]) {
        self.list.clear_render_target_view(handle, color, &[]);
    }
}
