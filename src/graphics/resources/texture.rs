use std::{
    fmt::Debug,
    ops::Deref,
    sync::{Arc, OnceLock},
};

use atomig::Atomic;
use oxidx::dx::{self, IDevice, IGraphicsCommandListExt};
use parking_lot::Mutex;

use crate::graphics::{
    command_queue::WorkerType,
    descriptor_heap::{DsvHeapView, ResourceDescriptor, RtvHeapView, SrvView, UavView},
    device::Device,
    heaps::{Allocation, MemoryHeap, MemoryHeapType},
    utils::TextureCopyableFootprints,
    worker_thread::WorkerThread,
};

use super::{
    staging_buffer::{StagingBuffer, StagingBufferDesc},
    GpuOnlyDescriptorAccess, NoGpuAccess, Resource, ResourceDesc, ResourceStates, SubresourceIndex,
    TextureResource, TextureResourceDesc, TextureUsage,
};

#[derive(Clone, Debug)]
pub struct Texture(Arc<TextureInner>);

impl Deref for Texture {
    type Target = TextureInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct TextureInner {
    raw: dx::Resource,
    desc: TextureDesc,
    state: Vec<Atomic<ResourceStates>>,
    allocation: Option<Allocation>,

    rtv: OnceLock<ResourceDescriptor<RtvHeapView>>,
    dsv: OnceLock<ResourceDescriptor<DsvHeapView>>,
    srv: OnceLock<ResourceDescriptor<SrvView>>,
    uav: OnceLock<ResourceDescriptor<UavView>>,

    // TODO: cached descriptors
    // cached_rtv: Mutex<HashMap<RtvDesc, ResourceDescriptor<RtvHeapView>>>
    // cached_dsv: Mutex<HashMap<DsvDesc, ResourceDescriptor<DsvHeapView>>>
    // cached_srv: Mutex<HashMap<SrvDesc, ResourceDescriptor<SrvView>>>
    // cached_uav: Mutex<HashMap<UavDesc, ResourceDescriptor<UavView>>>
    access: GpuOnlyDescriptorAccess,

    footprint: TextureCopyableFootprints,
    staging_buffer: StagingBuffer<u8>,
}

impl Texture {
    pub(in super::super) fn inner_new(
        device: &Device,
        resource: dx::Resource,
        desc: TextureDesc,
        access: GpuOnlyDescriptorAccess,
        state: ResourceStates,
        allocation: Option<Allocation>,
    ) -> Self {
        let footprint = device.get_texture_copyable_footprints(desc.clone());

        let state = (0..(desc.mip_levels * desc.count))
            .map(|_| Atomic::new(state))
            .collect();

        let staging_buffer = StagingBuffer::from_desc(
            device,
            StagingBufferDesc::new(footprint.total_size()),
            NoGpuAccess,
            ResourceStates::GenericRead,
        );

        Self(Arc::new(TextureInner {
            raw: resource,
            desc,
            state,
            allocation,
            rtv: Default::default(),
            dsv: Default::default(),
            srv: Default::default(),
            uav: Default::default(),
            access,
            staging_buffer,
            footprint,
        }))
    }
}

impl Texture {
    pub fn rtv(&self, desc: Option<&dx::RenderTargetViewDesc>) -> ResourceDescriptor<RtvHeapView> {
        match desc {
            Some(_desc) => todo!(),
            None => {
                let desc = self.rtv.get();
                if let Some(desc) = desc {
                    return *desc;
                }

                let handle = self.access.0.push_rtv(&self.raw, None);
                self.rtv.set(handle);

                handle
            }
        }
    }

    pub fn dsv(&self, desc: Option<&dx::DepthStencilViewDesc>) -> ResourceDescriptor<DsvHeapView> {
        match desc {
            Some(_desc) => todo!(),
            None => {
                let desc = self.dsv.get();
                if let Some(desc) = desc {
                    return *desc;
                }

                let handle = self.access.0.push_dsv(&self.raw, None);
                self.dsv.set(handle);

                handle
            }
        }
    }

    pub fn srv(&self, desc: Option<&dx::ShaderResourceViewDesc>) -> ResourceDescriptor<SrvView> {
        match desc {
            Some(_desc) => todo!(),
            None => {
                let desc = self.srv.get();
                if let Some(desc) = desc {
                    return *desc;
                }

                let handle = self.access.0.push_srv(&self.raw, None);
                self.srv.set(handle);

                handle
            }
        }
    }

    pub fn uav(&self, desc: Option<&dx::RenderTargetViewDesc>) -> ResourceDescriptor<UavView> {
        match desc {
            Some(_desc) => todo!(),
            None => {
                let desc = self.uav.get();
                if let Some(desc) = desc {
                    return *desc;
                }

                let handle = self.access.0.push_uav(&self.raw, None, None);
                self.uav.set(handle);

                handle
            }
        }
    }

    pub(in super::super) fn upload_data<WT: WorkerType>(
        &self,
        worker: &WorkerThread<WT>,
        src: &[u8],
    ) {
        let src = [dx::SubresourceData::new(src)];

        worker.list.update_subresources_fixed::<1, _, _>(
            &self.raw,
            self.staging_buffer.get_raw(),
            0,
            0..1,
            &src,
        );
    }
}

impl Drop for TextureInner {
    fn drop(&mut self) {
        if let Some(rtv) = self.rtv.get() {
            self.access.0.remove_rtv(*rtv);
        }

        if let Some(dsv) = self.dsv.get() {
            self.access.0.remove_dsv(*dsv);
        }

        if let Some(srv) = self.srv.get() {
            self.access.0.remove_srv(*srv);
        }

        if let Some(uav) = self.uav.get() {
            self.access.0.remove_uav(*uav);
        }
    }
}

impl Resource for Texture {
    type Desc = TextureDesc;
    type Access = GpuOnlyDescriptorAccess;

    fn get_raw(&self) -> &dx::Resource {
        &self.raw
    }

    fn get_desc(&self) -> Self::Desc {
        self.desc.clone()
    }

    fn from_desc(
        device: &Device,
        desc: Self::Desc,
        access: Self::Access,
        init_state: ResourceStates,
    ) -> Self {
        let resource: dx::Resource = device
            .raw
            .create_committed_resource(
                &dx::HeapProperties::default(),
                dx::HeapFlags::empty(),
                &desc.clone().into(),
                init_state.into(),
                desc.clear_color().as_ref(),
            )
            .unwrap();

        Self::inner_new(device, resource, desc, access, init_state, None)
    }

    fn from_raw_placed(
        heap: &MemoryHeap,
        raw: dx::Resource,
        desc: Self::Desc,
        access: Self::Access,
        state: ResourceStates,
        allocation: Allocation,
    ) -> Self {
        assert!(
            allocation.heap.mtype == MemoryHeapType::Gpu
                || allocation.heap.mtype == MemoryHeapType::Shared
        );

        Self::inner_new(&heap.device, raw, desc, access, state, Some(allocation))
    }
}

impl TextureResource for Texture {
    fn get_barrier(
        &self,
        state: ResourceStates,
        subresource: SubresourceIndex,
    ) -> Option<dx::ResourceBarrier<'_>> {
        let index =
            subresource.mip_index + subresource.array_index * (self.desc.mip_levels as usize);
        let old = self.state[index].swap(state, std::sync::atomic::Ordering::Relaxed);

        if old != state {
            Some(dx::ResourceBarrier::transition(
                self.get_raw(),
                old,
                state,
                Some(subresource as u32),
            ))
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextureDesc {
    width: u32,
    height: u32,
    count: u8,
    mip_levels: u8,
    format: dx::Format,
    layout: dx::TextureLayout,
    usage: TextureUsage,
    flags: dx::ResourceFlags,
}

impl TextureDesc {
    pub fn new(width: u32, height: u32, format: dx::Format) -> Self {
        Self {
            width,
            height,
            count: 1,
            mip_levels: 1,
            format,
            layout: dx::TextureLayout::RowMajor,
            usage: TextureUsage::ShaderResource,
            flags: dx::ResourceFlags::empty(),
        }
    }

    pub fn make_array(mut self, count: u8) -> Self {
        self.count = count;
        self
    }

    pub fn with_usage(mut self, usage: TextureUsage) -> Self {
        match &usage {
            TextureUsage::RenderTarget { .. } => {
                self.flags =
                    dx::ResourceFlags::AllowRenderTarget | dx::ResourceFlags::AllowUnorderedAccess
            }
            TextureUsage::DepthTarget { srv, .. } => {
                self.flags = dx::ResourceFlags::AllowDepthStencil;

                if !srv {
                    self.flags |= dx::ResourceFlags::DenyShaderResource
                }
            }
            TextureUsage::Storage => {
                self.flags = dx::ResourceFlags::AllowUnorderedAccess;
            }
            _ => {}
        }
        self.usage = usage;
        self
    }
}

impl Into<dx::ResourceDesc> for TextureDesc {
    fn into(self) -> dx::ResourceDesc {
        dx::ResourceDesc::texture_2d(self.width, self.height)
            .with_array_size(self.count as u16)
            .with_format(self.format)
            .with_flags(self.flags)
            .with_layout(self.layout)
            .with_mip_levels(self.mip_levels as u16)
    }
}

impl ResourceDesc for TextureDesc {}
impl TextureResourceDesc for TextureDesc {
    fn clear_color(&self) -> Option<dx::ClearValue> {
        match &self.usage {
            TextureUsage::RenderTarget { color } => {
                color.map(|v| dx::ClearValue::color(self.format, v))
            }
            TextureUsage::DepthTarget { color, .. } => {
                color.map(|v| dx::ClearValue::depth(self.format, v.0, v.1))
            }
            TextureUsage::ShaderResource => None,
            TextureUsage::Storage => None,
        }
    }

    fn with_layout(mut self, layout: dx::TextureLayout) -> Self {
        self.layout = layout;
        self
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::Texture;

    const fn is_send_sync<T: Send + Sync>() {}

    const _: () = is_send_sync::<Texture>();
}