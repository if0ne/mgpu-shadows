use super::{
    command_allocator::CommandAllocator,
    command_queue::{Graphics, WorkerType},
    device::Device,
    resources::{Resource, SharedResource},
};

use oxidx::dx::{self, IDevice, IGraphicsCommandList};

#[derive(Debug)]
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

    pub fn pull_shared<R: Resource>(&self, shared_resource: &SharedResource<R>) {
        if self.device.is_cross_adapter_texture_supported() {
            return;
        }

        let local_state = shared_resource
            .local_resource()
            .set_current_state(dx::ResourceStates::CopyDest);
        let shared_state = shared_resource
            .cross_resource()
            .set_current_state(dx::ResourceStates::CopySource);

        self.list.resource_barrier(&[
            dx::ResourceBarrier::transition(
                shared_resource.local_resource().get_raw(),
                local_state,
                dx::ResourceStates::CopyDest,
            ),
            dx::ResourceBarrier::transition(
                shared_resource.cross_resource().get_raw(),
                shared_state,
                dx::ResourceStates::CopySource,
            ),
        ]);

        self.list.copy_resource(
            shared_resource.local_resource().get_raw(),
            shared_resource.cross_resource().get_raw(),
        );
    }

    pub fn push_shared<R: Resource>(&self, shared_resource: &SharedResource<R>) {
        if self.device.is_cross_adapter_texture_supported() {
            return;
        }

        let shared_state = shared_resource
            .cross_resource()
            .set_current_state(dx::ResourceStates::CopyDest);
        let local_state = shared_resource
            .local_resource()
            .set_current_state(dx::ResourceStates::CopySource);

        self.list.resource_barrier(&[
            dx::ResourceBarrier::transition(
                shared_resource.cross_resource().get_raw(),
                shared_state,
                dx::ResourceStates::CopyDest,
            ),
            dx::ResourceBarrier::transition(
                shared_resource.local_resource().get_raw(),
                local_state,
                dx::ResourceStates::CopySource,
            ),
        ]);

        self.list.copy_resource(
            shared_resource.cross_resource().get_raw(),
            shared_resource.local_resource().get_raw(),
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
