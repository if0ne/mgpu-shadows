#![allow(private_bounds)]
#![allow(private_interfaces)]

use std::{num::NonZero, ops::Deref, path::Path, sync::Arc};

use oxidx::dx::{self, IAdapter3, IDevice};

use super::{
    commands::{CommandAllocator, CommandQueue, Compute, Direct, Transfer, WorkerType},
    fence::{Fence, LocalFence, SharedFence},
    heaps::MemoryHeap,
    queries::{QueryHeap, QueryHeapType},
    resources::{
        BufferResource, BufferResourceDesc, ImageResource, ImageResourceDesc, Resource,
        ShareableBuffer, ShareableImage, SharedResource,
    },
    swapchain::Swapchain,
    types::{
        BufferCopyableFootprints, MemoryHeapType, MipInfo, SwapchainDesc, TextureCopyableFootprints,
    },
    views::ViewAllocator,
    BindingType, Graphics, GraphicsPipelineDesc, Pipeline, PipelineLayout, Pixel, ResourceStates,
    Sampler, SamplerDesc, Shader, ShaderType, StaticSampler, Vertex,
};

#[derive(Clone, Debug)]
pub struct Device(Arc<DeviceInner>);

impl Device {
    pub fn new(factory: dx::Factory4, adapter: dx::Adapter3) -> Self {
        let name = adapter.get_desc1().unwrap().description().to_string();

        let raw: dx::Device = dx::create_device(Some(&adapter), dx::FeatureLevel::Level11).unwrap();

        let mut feature = dx::features::OptionsFeature::default();
        raw.check_feature_support(&mut feature).unwrap();

        Self(Arc::new(DeviceInner {
            name,
            factory,
            adapter,
            raw,
            is_cross_adapter_texture_supported: feature.cross_adapter_row_major_texture_supported(),
        }))
    }
}

impl Deref for Device {
    type Target = DeviceInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct DeviceInner {
    name: String,
    pub(super) factory: dx::Factory4,
    adapter: dx::Adapter3,
    pub(super) raw: dx::Device,

    is_cross_adapter_texture_supported: bool,
}

impl Device {
    pub(super) fn create_command_allocator<T: WorkerType>(&self) -> CommandAllocator<T> {
        CommandAllocator::inner_new(&self.raw, T::RAW_TYPE)
    }
}

impl Device {
    pub fn create_graphics_command_queue(&self, fence: Fence) -> CommandQueue<Direct> {
        CommandQueue::inner_new(self.clone(), fence)
    }

    pub fn create_compute_command_queue(&self, fence: Fence) -> CommandQueue<Compute> {
        CommandQueue::inner_new(self.clone(), fence)
    }

    pub fn create_transfer_command_queue(&self, fence: Fence) -> CommandQueue<Transfer> {
        CommandQueue::inner_new(self.clone(), fence)
    }

    pub fn create_descriptor_allocator(
        &self,
        rtv_size: usize,
        dsv_size: usize,
        cbv_srv_uav_size: usize,
        sampler_size: usize,
    ) -> ViewAllocator {
        ViewAllocator::inner_new(self, rtv_size, dsv_size, cbv_srv_uav_size, sampler_size)
    }

    pub fn create_fence(&self) -> LocalFence {
        LocalFence::inner_new(self)
    }

    pub fn create_shared_fence(&self) -> SharedFence {
        SharedFence::inner_new(self.clone())
    }

    pub fn create_heap(&self, size: usize, mtype: MemoryHeapType) -> MemoryHeap {
        MemoryHeap::inner_new(self.clone(), size, mtype)
    }

    pub fn create_commited_resource<R: Resource>(
        &self,
        desc: R::Desc,
        access: R::Access,
        init_state: ResourceStates,
    ) -> R {
        R::from_desc(self, desc, access, init_state)
    }

    pub fn create_placed_buffer<R: BufferResource>(
        &self,
        heap: &MemoryHeap,
        desc: R::Desc,
        offset: usize,
        access: R::Access,
        initial_state: ResourceStates,
    ) -> R {
        heap.create_placed_buffer(desc, offset, access, initial_state)
    }

    pub fn create_placed_image<R: ImageResource>(
        &self,
        heap: &MemoryHeap,
        desc: R::Desc,
        offset: usize,
        access: R::Access,
        initial_state: ResourceStates,
    ) -> R {
        heap.create_placed_texture(desc, offset, access, initial_state)
    }

    pub fn create_shared_buffer<R: ShareableBuffer>(
        &self,
        heap: &MemoryHeap,
        offset: usize,
        desc: R::Desc,
        access: R::Access,
        local_state: ResourceStates,
        share_state: ResourceStates,
    ) -> SharedResource<R> {
        SharedResource::inner_new_buffer(heap, offset, desc, access, local_state, share_state)
    }

    pub fn create_shared_image<R: ShareableImage>(
        &self,
        heap: &MemoryHeap,
        offset: usize,
        desc: R::Desc,
        access: R::Access,
        local_state: ResourceStates,
        share_state: ResourceStates,
    ) -> SharedResource<R> {
        SharedResource::inner_new_image(heap, offset, desc, access, local_state, share_state)
    }

    pub fn create_swapchain(
        &self,
        queue: CommandQueue<Direct>,
        descriptor_allocator: ViewAllocator,
        hwnd: NonZero<isize>,
        desc: SwapchainDesc,
    ) -> Swapchain {
        Swapchain::inner_new(self.clone(), queue, descriptor_allocator, hwnd, desc)
    }

    pub fn create_query_heap<T: QueryHeapType>(&self, count: usize) -> QueryHeap<T> {
        QueryHeap::inner_new(self, count)
    }

    pub fn is_cross_adapter_texture_supported(&self) -> bool {
        self.is_cross_adapter_texture_supported
    }

    pub fn create_pipeline_layout(
        &self,
        layout: &[BindingType],
        static_samplers: &[StaticSampler],
    ) -> PipelineLayout {
        PipelineLayout::inner_new(self, layout, static_samplers)
    }

    pub fn create_sampler(&self, allocator: ViewAllocator, desc: &SamplerDesc) -> Sampler {
        Sampler::inner_new(allocator, desc)
    }

    pub fn create_shader<T: ShaderType>(
        &self,
        path: impl AsRef<Path>,
        entry_point: impl AsRef<str>,
        defines: &[(&'static str, &'static str)],
    ) -> Shader<T> {
        Shader::inner_new(path, entry_point, defines)
    }

    pub fn create_graphics_pipeline(&self, desc: &GraphicsPipelineDesc) -> Pipeline<Graphics> {
        Pipeline::inner_new_graphics(self, desc)
    }

    pub fn create_vertex_shader(
        &self,
        path: impl AsRef<Path>,
        entry_point: impl AsRef<str>,
        defines: &[(&'static str, &'static str)],
    ) -> Shader<Vertex> {
        Shader::inner_new(path, entry_point, defines)
    }

    pub fn create_pixel_shader(
        &self,
        path: impl AsRef<Path>,
        entry_point: impl AsRef<str>,
        defines: &[(&'static str, &'static str)],
    ) -> Shader<Pixel> {
        Shader::inner_new(path, entry_point, defines)
    }

    pub fn get_buffer_copyable_footprints<T: BufferResourceDesc>(
        &self,
        desc: T,
    ) -> BufferCopyableFootprints {
        let mut layouts = [Default::default(); 1];
        let mut num_rows = [Default::default(); 1];
        let mut row_sizes = [Default::default(); 1];

        let total_size = self.raw.get_copyable_footprints(
            &desc.into(),
            0..1,
            0,
            &mut layouts,
            &mut num_rows,
            &mut row_sizes,
        );

        BufferCopyableFootprints::new(total_size as usize)
    }

    pub fn get_texture_copyable_footprints<T: ImageResourceDesc>(
        &self,
        desc: T,
    ) -> TextureCopyableFootprints {
        let desc: dx::ResourceDesc = desc.into();

        let sub_count = if desc.dimension() == dx::ResourceDimension::Texture3D {
            desc.mip_levels() as usize
        } else {
            (desc.depth_or_array_size() * desc.mip_levels()) as usize
        };

        let mut layouts = vec![Default::default(); sub_count];
        let mut num_rows = vec![Default::default(); sub_count];
        let mut row_sizes = vec![Default::default(); sub_count];

        let total_size = self.raw.get_copyable_footprints(
            &desc,
            0..(sub_count as u32),
            0,
            &mut layouts,
            &mut num_rows,
            &mut row_sizes,
        );

        let subresources = (0..sub_count)
            .map(|i| MipInfo {
                width: layouts[i].footprint().width(),
                height: layouts[i].footprint().height(),
                depth: layouts[i].footprint().depth(),
                row_size: row_sizes[i] as usize,
                size: num_rows[i] as usize * row_sizes[i] as usize,
            })
            .collect();

        TextureCopyableFootprints::new(
            total_size as usize,
            desc.mip_levels() as usize,
            subresources,
        )
    }
}
