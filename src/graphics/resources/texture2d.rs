use std::{fmt::Debug, ops::Deref, sync::Arc};

use oxidx::dx::{self, IDevice, IGraphicsCommandListExt};
use parking_lot::Mutex;

use crate::graphics::{
    command_queue::WorkerType,
    descriptor_heap::{DsvHeapView, ResourceDescriptor, RtvHeapView, SrvView, UavView},
    device::Device,
    heaps::{Allocation, MemoryHeap, MemoryHeapType},
    worker_thread::WorkerThread,
};

use super::{
    staging_buffer::{StagingBuffer, StagingBufferDesc},
    GpuOnlyDescriptorAccess, NoGpuAccess, Resource, ResourceDesc,
};

#[derive(Clone, Debug)]
pub struct Texture2D(Arc<Texture2DInner>);

impl Deref for Texture2D {
    type Target = Texture2DInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct Texture2DInner {
    raw: dx::Resource,
    desc: Texture2DDesc,
    state: Vec<Mutex<dx::ResourceStates>>,
    allocation: Option<Allocation>,

    rtv: Mutex<Option<ResourceDescriptor<RtvHeapView>>>,
    dsv: Mutex<Option<ResourceDescriptor<DsvHeapView>>>,
    srv: Mutex<Option<ResourceDescriptor<SrvView>>>,
    uav: Mutex<Option<ResourceDescriptor<UavView>>>,

    // TODO: cached descriptors
    // cached_rtv: Mutex<HashMap<RtvDesc, ResourceDescriptor<RtvHeapView>>>
    // cached_dsv: Mutex<HashMap<DsvDesc, ResourceDescriptor<DsvHeapView>>>
    // cached_srv: Mutex<HashMap<SrvDesc, ResourceDescriptor<SrvView>>>
    // cached_uav: Mutex<HashMap<UavDesc, ResourceDescriptor<UavView>>>
    access: GpuOnlyDescriptorAccess,

    staging_buffer: StagingBuffer<u8>,
}

impl Texture2D {
    pub(in super::super) fn inner_new(
        device: &Device,
        resource: dx::Resource,
        desc: Texture2DDesc,
        access: GpuOnlyDescriptorAccess,
        state: dx::ResourceStates,
        allocation: Option<Allocation>,
    ) -> Self {
        let raw_desc = desc.clone().into();

        let mut layouts = [Default::default(); 1];
        let mut num_rows = [Default::default(); 1];
        let mut row_sizes = [Default::default(); 1];

        let total_size = device.raw.get_copyable_footprints(
            &raw_desc,
            0..1,
            0,
            &mut layouts,
            &mut num_rows,
            &mut row_sizes,
        );

        let state = (0..desc.mip_levels).map(|_| Mutex::new(state)).collect();

        let staging_buffer = StagingBuffer::from_desc(
            device,
            StagingBufferDesc::new(total_size as usize),
            NoGpuAccess,
            dx::ResourceStates::GenericRead,
            None,
        );

        Self(Arc::new(Texture2DInner {
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
        }))
    }
}

impl Texture2D {
    pub fn rtv(&self, desc: Option<&dx::RenderTargetViewDesc>) -> ResourceDescriptor<RtvHeapView> {
        match desc {
            Some(_desc) => todo!(),
            None => {
                let mut guard = self.rtv.lock();
                if let Some(desc) = *guard {
                    return desc;
                }

                let handle = self.access.0.push_rtv(&self.raw, None);
                *guard = Some(handle);

                handle
            }
        }
    }

    pub fn dsv(&self, desc: Option<&dx::DepthStencilViewDesc>) -> ResourceDescriptor<DsvHeapView> {
        match desc {
            Some(_desc) => todo!(),
            None => {
                let mut guard = self.dsv.lock();
                if let Some(desc) = *guard {
                    return desc;
                }

                let handle = self.access.0.push_dsv(&self.raw, None);
                *guard = Some(handle);

                handle
            }
        }
    }

    pub fn srv(&self, desc: Option<&dx::ShaderResourceViewDesc>) -> ResourceDescriptor<SrvView> {
        match desc {
            Some(_desc) => todo!(),
            None => {
                let mut guard = self.srv.lock();
                if let Some(desc) = *guard {
                    return desc;
                }

                let handle = self.access.0.push_srv(&self.raw, None);
                *guard = Some(handle);

                handle
            }
        }
    }

    pub fn uav(&self, desc: Option<&dx::RenderTargetViewDesc>) -> ResourceDescriptor<UavView> {
        match desc {
            Some(_desc) => todo!(),
            None => {
                let mut guard = self.uav.lock();
                if let Some(desc) = *guard {
                    return desc;
                }

                let handle = self.access.0.push_uav(&self.raw, None, None);
                *guard = Some(handle);

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

impl Drop for Texture2DInner {
    fn drop(&mut self) {
        if let Some(rtv) = *self.rtv.lock() {
            self.access.0.remove_rtv(rtv);
        }

        if let Some(dsv) = *self.dsv.lock() {
            self.access.0.remove_dsv(dsv);
        }

        if let Some(srv) = *self.srv.lock() {
            self.access.0.remove_srv(srv);
        }

        if let Some(uav) = *self.uav.lock() {
            self.access.0.remove_uav(uav);
        }
    }
}

impl Resource for Texture2D {
    type Desc = Texture2DDesc;
    type Access = GpuOnlyDescriptorAccess;

    fn get_raw(&self) -> &dx::Resource {
        &self.raw
    }

    fn get_barrier(
        &self,
        state: dx::ResourceStates,
        subresource: usize,
    ) -> Option<dx::ResourceBarrier<'_>> {
        let mut guard = self.state[subresource].lock();
        let old = *guard;
        *guard = state;

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

    fn get_desc(&self) -> Self::Desc {
        self.desc.clone()
    }

    fn from_desc(
        device: &Device,
        desc: Self::Desc,
        access: Self::Access,
        init_state: dx::ResourceStates,
        clear_color: Option<&dx::ClearValue>,
    ) -> Self {
        let resource: dx::Resource = device
            .raw
            .create_committed_resource(
                &dx::HeapProperties::default(),
                dx::HeapFlags::empty(),
                &desc.clone().into(),
                init_state,
                clear_color,
            )
            .unwrap();

        Self::inner_new(device, resource, desc, access, init_state, None)
    }

    fn from_raw_placed(
        heap: &MemoryHeap,
        raw: dx::Resource,
        desc: Self::Desc,
        access: Self::Access,
        state: dx::ResourceStates,
        allocation: Allocation,
    ) -> Self {
        assert!(allocation.heap.mtype == MemoryHeapType::Gpu || allocation.heap.mtype == MemoryHeapType::Shared);

        Self::inner_new(&heap.device, raw, desc, access, state, Some(allocation))
    }
}

#[derive(Clone, Debug)]
pub struct Texture2DDesc {
    pub width: u32,
    pub height: u32,
    pub format: dx::Format,
    pub flags: dx::ResourceFlags,
    pub layout: dx::TextureLayout,
    pub mip_levels: u16,
}

impl Texture2DDesc {}

impl Into<dx::ResourceDesc> for Texture2DDesc {
    fn into(self) -> dx::ResourceDesc {
        dx::ResourceDesc::texture_2d(self.width, self.height)
            .with_format(self.format)
            .with_flags(self.flags)
            .with_layout(self.layout)
            .with_mip_levels(self.mip_levels)
    }
}

impl ResourceDesc for Texture2DDesc {
    fn flags(&self) -> dx::ResourceFlags {
        self.flags
    }

    fn with_flags(mut self, flags: dx::ResourceFlags) -> Self {
        self.flags = flags;
        self
    }

    fn with_layout(mut self, layout: dx::TextureLayout) -> Self {
        self.layout = layout;
        self
    }
}
