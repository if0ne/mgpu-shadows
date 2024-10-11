use super::{
    command_allocator::CommandAllocator,
    command_queue::{Graphics, WorkerType},
    device::Device,
    resources::{
        BufferResource, Image, ImageResource, IndexBuffer, IndexBufferType, ResourceStates,
        SharedResource, SubresourceIndex, VertexBuffer,
    },
};

use oxidx::dx::{self, IDevice, IGraphicsCommandList};
use smallvec::SmallVec;

#[derive(Debug)]
pub struct WorkerThread<T: WorkerType> {
    pub(super) device: Device,
    pub(super) frequency: f64,
    pub(super) allocator: CommandAllocator<T>,
    pub(super) list: dx::GraphicsCommandList,
}

impl<T: WorkerType> WorkerThread<T> {
    fn inner_new(
        device: Device,
        allocator: CommandAllocator<T>,
        r#type: dx::CommandListType,
        frequency: f64,
    ) -> Self {
        let list = device
            .raw
            .create_command_list(0, r#type, &allocator.raw, dx::PSO_NONE)
            .unwrap();

        Self {
            device,
            list,
            allocator,
            frequency,
        }
    }

    pub fn pull_shared_texture<R: ImageResource>(&self, shared_resource: &SharedResource<R>) {
        if self.device.is_cross_adapter_texture_supported() {
            return;
        }

        let mut barriers: SmallVec<[dx::ResourceBarrier<'_>; 2]> = Default::default();

        if let Some(shared_state) = shared_resource.cross_resource().get_barrier(
            ResourceStates::CopySrc,
            SubresourceIndex {
                array_index: 0,
                mip_index: 0,
            },
        ) {
            barriers.push(shared_state);
        }
        if let Some(local_state) = shared_resource.local_resource().get_barrier(
            ResourceStates::CopyDst,
            SubresourceIndex {
                array_index: 0,
                mip_index: 0,
            },
        ) {
            barriers.push(local_state);
        }

        self.barrier(&barriers);

        self.list.copy_resource(
            shared_resource.local_resource().get_raw(),
            shared_resource.cross_resource().get_raw(),
        );
    }

    pub fn push_shared_texture<R: ImageResource>(&self, shared_resource: &SharedResource<R>) {
        if self.device.is_cross_adapter_texture_supported() {
            return;
        }

        let mut barriers: SmallVec<[dx::ResourceBarrier<'_>; 2]> = Default::default();

        if let Some(shared_state) = shared_resource.cross_resource().get_barrier(
            ResourceStates::CopyDst,
            SubresourceIndex {
                array_index: 0,
                mip_index: 0,
            },
        ) {
            barriers.push(shared_state);
        }
        if let Some(local_state) = shared_resource.local_resource().get_barrier(
            ResourceStates::CopySrc,
            SubresourceIndex {
                array_index: 0,
                mip_index: 0,
            },
        ) {
            barriers.push(local_state);
        }

        self.barrier(&barriers);

        self.list.copy_resource(
            shared_resource.cross_resource().get_raw(),
            shared_resource.local_resource().get_raw(),
        );
    }

    pub fn pull_shared_buffer<R: BufferResource>(&self, shared_resource: &SharedResource<R>) {
        if self.device.is_cross_adapter_texture_supported() {
            return;
        }

        let mut barriers: SmallVec<[dx::ResourceBarrier<'_>; 2]> = Default::default();

        if let Some(shared_state) = shared_resource
            .cross_resource()
            .get_barrier(ResourceStates::CopySrc)
        {
            barriers.push(shared_state);
        }
        if let Some(local_state) = shared_resource
            .local_resource()
            .get_barrier(ResourceStates::CopyDst)
        {
            barriers.push(local_state);
        }

        self.barrier(&barriers);

        self.list.copy_resource(
            shared_resource.local_resource().get_raw(),
            shared_resource.cross_resource().get_raw(),
        );
    }

    pub fn push_shared_buffer<R: BufferResource>(&self, shared_resource: &SharedResource<R>) {
        if self.device.is_cross_adapter_texture_supported() {
            return;
        }

        let mut barriers: SmallVec<[dx::ResourceBarrier<'_>; 2]> = Default::default();

        if let Some(shared_state) = shared_resource
            .cross_resource()
            .get_barrier(ResourceStates::CopyDst)
        {
            barriers.push(shared_state);
        }
        if let Some(local_state) = shared_resource
            .local_resource()
            .get_barrier(ResourceStates::CopySrc)
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

    pub fn upload_to_vertex_buffer<VT: Clone + Copy>(&self, dst: &VertexBuffer<VT>, src: &[VT]) {
        if let Some(barrier) = dst.get_barrier(ResourceStates::CopyDst) {
            self.barrier(&[barrier]);
        }

        dst.upload_data(self, src);

        if let Some(barrier) = dst.get_barrier(ResourceStates::GenericRead) {
            self.barrier(&[barrier]);
        }
    }

    pub fn upload_to_index_buffer<IT: IndexBufferType>(
        &self,
        dst: &IndexBuffer<IT>,
        src: &[IT::Raw],
    ) {
        if let Some(barrier) = dst.get_barrier(ResourceStates::CopyDst) {
            self.barrier(&[barrier]);
        }

        dst.upload_data(self, src);

        if let Some(barrier) = dst.get_barrier(ResourceStates::GenericRead) {
            self.barrier(&[barrier]);
        }
    }

    pub fn upload_to_texture2d(&self, dst: &Image, src: &[u8]) {
        if let Some(barrier) = dst.get_barrier(
            ResourceStates::CopyDst,
            SubresourceIndex {
                array_index: 0,
                mip_index: 0,
            },
        ) {
            self.barrier(&[barrier]);
        }

        dst.upload_data(self, src);

        // TODO: Return in prev state?
        if let Some(barrier) = dst.get_barrier(
            ResourceStates::Common,
            SubresourceIndex {
                array_index: 0,
                mip_index: 0,
            },
        ) {
            self.barrier(&[barrier]);
        }
    }
}

impl WorkerThread<Graphics> {
    pub fn clear_rt(&self, handle: dx::CpuDescriptorHandle, color: [f32; 4]) {
        self.list.clear_render_target_view(handle, color, &[]);
    }

    pub fn bind_vertex_buffer(&self, slot: u32, view: dx::VertexBufferView) {
        self.list.ia_set_vertex_buffers(slot, &[view]);
    }

    pub fn bind_vertex_buffers(&self, slot: u32, views: &[dx::VertexBufferView]) {
        self.list.ia_set_vertex_buffers(slot, views);
    }

    pub fn bind_index_buffer(&self, view: dx::IndexBufferView) {
        self.list.ia_set_index_buffer(Some(&view));
    }
}
