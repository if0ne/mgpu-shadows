#![allow(private_bounds)]

use super::{
    command_allocator::CommandAllocator, command_queue::WorkerType, device::Device,
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
        if shared_resource.owner().is_cross_adapter_texture_supported() {
            return;
        }

        let desc = shared_resource.get_desc();

        let mut layouts = [dx::PlacedSubresourceFootprint::default(); 1];
        let mut num_rows = [0; 1];
        let mut row_sizes = [0; 1];
        self.device.raw.get_copyable_footprints(
            &desc,
            0..1,
            0,
            &mut layouts,
            &mut num_rows,
            &mut row_sizes,
        );

        let dest =
            dx::TextureCopyLocation::placed_footprint(shared_resource.cross_resource(), layouts[0]);
        let src = dx::TextureCopyLocation::subresource(shared_resource.local_resource(), 0);
        let src_box = dx::DxBox::default()
            .with_right(desc.width() as u32)
            .with_bottom(desc.height() as u32);

        self.list
            .copy_texture_region(&dest, 0, 0, 0, &src, Some(&src_box));
    }

    pub fn push_shared(&self, shared_resource: &SharedResource) {
        if shared_resource.owner().is_cross_adapter_texture_supported() {
            return;
        }

        let desc = shared_resource.get_desc();

        let mut layouts = [dx::PlacedSubresourceFootprint::default(); 1];
        let mut num_rows = [0; 1];
        let mut row_sizes = [0; 1];
        self.device.raw.get_copyable_footprints(
            &desc,
            0..1,
            0,
            &mut layouts,
            &mut num_rows,
            &mut row_sizes,
        );

        let dest =
            dx::TextureCopyLocation::placed_footprint(shared_resource.local_resource(), layouts[0]);
        let src = dx::TextureCopyLocation::subresource(shared_resource.cross_resource(), 0);
        let src_box = dx::DxBox::default()
            .with_right(desc.width() as u32)
            .with_bottom(desc.height() as u32);

        self.list
            .copy_texture_region(&dest, 0, 0, 0, &src, Some(&src_box));
    }
}
