use super::{
    command_allocator::CommandAllocator,
    command_queue::{Graphics, WorkerType},
    device::Device,
    resources::{Resource, SharedResource, VertexBuffer},
};

use oxidx::dx::{self, IDevice, IGraphicsCommandList};
use smallvec::SmallVec;

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

        let mut barriers: SmallVec<[dx::ResourceBarrier<'_>; 2]> = Default::default();

        if let Some(shared_state) = shared_resource
            .cross_resource()
            .get_barrier(dx::ResourceStates::CopySource)
        {
            barriers.push(shared_state);
        }
        if let Some(local_state) = shared_resource
            .local_resource()
            .get_barrier(dx::ResourceStates::CopyDest)
        {
            barriers.push(local_state);
        }

        self.barrier(&barriers);

        self.list.copy_resource(
            shared_resource.local_resource().get_raw(),
            shared_resource.cross_resource().get_raw(),
        );
    }

    pub fn push_shared<R: Resource>(&self, shared_resource: &SharedResource<R>) {
        if self.device.is_cross_adapter_texture_supported() {
            return;
        }

        let mut barriers: SmallVec<[dx::ResourceBarrier<'_>; 2]> = Default::default();

        if let Some(shared_state) = shared_resource
            .cross_resource()
            .get_barrier(dx::ResourceStates::CopyDest)
        {
            barriers.push(shared_state);
        }
        if let Some(local_state) = shared_resource
            .local_resource()
            .get_barrier(dx::ResourceStates::CopySource)
        {
            barriers.push(local_state);
        }

        self.barrier(&barriers);

        self.list.copy_resource(
            shared_resource.cross_resource().get_raw(),
            shared_resource.local_resource().get_raw(),
        );
    }

    // TODO: Batched barrier
    pub fn barrier(&self, barriers: &[dx::ResourceBarrier<'_>]) {
        self.list.resource_barrier(barriers);
    }

    pub fn upload_to_vertex_buffer<VT: Clone>(&self, dst: &VertexBuffer<VT>, src: &[VT]) {
        if let Some(barrier) = dst.get_barrier(dx::ResourceStates::CopyDest) {
            self.barrier(&[barrier]);
        }

        dst.upload_data(self, src);

        if let Some(barrier) = dst.get_barrier(dx::ResourceStates::GenericRead) {
            self.barrier(&[barrier]);
        }
    }
}

impl WorkerThread<Graphics> {
    pub fn clear_rt(&self, handle: dx::CpuDescriptorHandle, color: [f32; 4]) {
        self.list.clear_render_target_view(handle, color, &[]);
    }
}
