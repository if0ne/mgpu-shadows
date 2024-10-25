use std::{
    collections::HashMap,
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, Range},
    sync::{Arc, OnceLock},
};

use atomig::Atomic;
use oxidx::dx::{self, IDevice, IGraphicsCommandListExt};
use parking_lot::Mutex;

use crate::graphics::{
    commands::{WorkerThread, WorkerType},
    device::Device,
    heaps::{Allocation, MemoryHeap},
    types::{MemoryHeapType, SubresourceIndex, TextureCopyableFootprints, TextureUsage},
    views::{DsvView, GpuView, RtvView, SrvView, UavView, ViewType},
    ResourceStates,
};

use super::{
    staging_buffer::{StagingBuffer, StagingBufferDesc},
    ImageResource, ImageResourceDesc, NoGpuAccess, Resource, ResourceDesc, ShareableImage,
    ShareableImageDesc, ViewAccess,
};

#[derive(Clone, Debug)]
pub struct Image(Arc<ImageInner>);

impl Deref for Image {
    type Target = ImageInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct ImageInner {
    raw: dx::Resource,
    desc: ImageDesc,
    state: Vec<Atomic<ResourceStates>>,
    allocation: Option<Allocation>,

    rtv: OnceLock<GpuView<RtvView>>,
    dsv: OnceLock<GpuView<DsvView>>,
    srv: OnceLock<GpuView<SrvView>>,
    uav: OnceLock<GpuView<UavView>>,

    cached_rtv: Mutex<HashMap<ImageViewDesc<RtvView>, GpuView<RtvView>>>,
    cached_dsv: Mutex<HashMap<ImageViewDesc<DsvView>, GpuView<DsvView>>>,
    cached_srv: Mutex<HashMap<ImageViewDesc<SrvView>, GpuView<SrvView>>>,
    cached_uav: Mutex<HashMap<ImageViewDesc<UavView>, GpuView<UavView>>>,
    access: ViewAccess,

    footprint: TextureCopyableFootprints,
    staging_buffer: StagingBuffer<u8>,
}

impl Image {
    pub(in super::super) fn inner_new(
        device: &Device,
        resource: dx::Resource,
        desc: ImageDesc,
        access: ViewAccess,
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

        Self(Arc::new(ImageInner {
            raw: resource,
            desc,
            state,
            allocation,
            rtv: Default::default(),
            dsv: Default::default(),
            srv: Default::default(),
            uav: Default::default(),
            cached_rtv: Default::default(),
            cached_dsv: Default::default(),
            cached_srv: Default::default(),
            cached_uav: Default::default(),
            access,
            staging_buffer,
            footprint,
        }))
    }
}

// TODO: Desc validation
impl Image {
    pub fn rtv(&self, desc: Option<ImageViewDesc<RtvView>>) -> GpuView<RtvView> {
        assert!(self.is_support_rtv());

        match desc {
            Some(mut desc) => {
                assert!(self.is_array() || !desc.is_array());
                assert!(
                    desc.mip_slice <= self.desc.mip_levels && desc.mip_base < self.desc.mip_levels
                );
                assert!(desc.array.as_ref().unwrap_or(&(0..0)).end < self.desc.count);

                if desc.format.is_none() {
                    desc.format = Some(self.desc.format);
                }

                *self
                    .cached_rtv
                    .lock()
                    .entry(desc.clone())
                    .or_insert_with(|| self.access.0.push_rtv(&self.raw, Some(&desc.into())))
            }
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

    pub fn dsv(&self, desc: Option<ImageViewDesc<DsvView>>) -> GpuView<DsvView> {
        assert!(self.is_support_dsv());

        match desc {
            Some(mut desc) => {
                assert!(self.is_array() || !desc.is_array());
                assert!(
                    desc.mip_slice <= self.desc.mip_levels && desc.mip_base < self.desc.mip_levels
                );
                assert!(desc.array.as_ref().unwrap_or(&(0..0)).end < self.desc.count);

                if desc.format.is_none() {
                    desc.format = Some(self.desc.format);
                }

                *self
                    .cached_dsv
                    .lock()
                    .entry(desc.clone())
                    .or_insert_with(|| self.access.0.push_dsv(&self.raw, Some(&desc.into())))
            }
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

    pub fn srv(&self, desc: Option<ImageViewDesc<SrvView>>) -> GpuView<SrvView> {
        assert!(self.is_support_srv());

        match desc {
            Some(mut desc) => {
                assert!(self.is_array() || !desc.is_array());
                assert!(
                    desc.mip_slice <= self.desc.mip_levels && desc.mip_base < self.desc.mip_levels
                );
                assert!(desc.array.as_ref().unwrap_or(&(0..0)).end < self.desc.count);

                if desc.format.is_none() {
                    desc.format = Some(self.desc.format);
                }

                *self
                    .cached_srv
                    .lock()
                    .entry(desc.clone())
                    .or_insert_with(|| self.access.0.push_srv(&self.raw, Some(&desc.into())))
            }
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

    pub fn uav(&self, desc: Option<ImageViewDesc<UavView>>) -> GpuView<UavView> {
        assert!(self.is_support_uav());

        match desc {
            Some(mut desc) => {
                assert!(self.is_array() || !desc.is_array());
                assert!(
                    desc.mip_slice <= self.desc.mip_levels && desc.mip_base < self.desc.mip_levels
                );
                assert!(desc.array.as_ref().unwrap_or(&(0..0)).end < self.desc.count);

                if desc.format.is_none() {
                    desc.format = Some(self.desc.format);
                }

                *self
                    .cached_uav
                    .lock()
                    .entry(desc.clone())
                    .or_insert_with(|| self.access.0.push_uav(&self.raw, None, Some(&desc.into())))
            }
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

    pub fn is_array(&self) -> bool {
        self.desc.count != 1
    }

    pub fn is_support_rtv(&self) -> bool {
        self.desc
            .flags
            .intersects(dx::ResourceFlags::AllowRenderTarget)
    }

    pub fn is_support_dsv(&self) -> bool {
        self.desc
            .flags
            .intersects(dx::ResourceFlags::AllowDepthStencil)
    }

    pub fn is_support_uav(&self) -> bool {
        self.desc
            .flags
            .intersects(dx::ResourceFlags::AllowUnorderedAccess)
    }

    pub fn is_support_srv(&self) -> bool {
        !self
            .desc
            .flags
            .intersects(dx::ResourceFlags::DenyShaderResource)
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

impl Drop for ImageInner {
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

        self.cached_rtv
            .lock()
            .iter()
            .for_each(|(_, handle)| self.access.0.remove_rtv(*handle));
        self.cached_dsv
            .lock()
            .iter()
            .for_each(|(_, handle)| self.access.0.remove_dsv(*handle));
        self.cached_srv
            .lock()
            .iter()
            .for_each(|(_, handle)| self.access.0.remove_srv(*handle));
        self.cached_uav
            .lock()
            .iter()
            .for_each(|(_, handle)| self.access.0.remove_uav(*handle));
    }
}

impl Resource for Image {
    type Desc = ImageDesc;
    type Access = ViewAccess;

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
                init_state.as_raw(),
                desc.clear_color().as_ref(),
            )
            .unwrap();

        Self::inner_new(device, resource, desc, access, init_state, None)
    }

    fn from_raw_placed(
        heap: &MemoryHeap,
        desc: Self::Desc,
        access: Self::Access,
        state: ResourceStates,
        allocation: Allocation,
    ) -> Self {
        assert!(
            allocation.heap.mtype == MemoryHeapType::Gpu
                || allocation.heap.mtype == MemoryHeapType::Shared
        );

        let raw_desc = desc.clone().into();

        let raw = heap
            .device
            .raw
            .create_placed_resource(
                &heap.heap,
                allocation.offset,
                &raw_desc,
                state.as_raw(),
                desc.clear_color().as_ref(),
            )
            .unwrap();

        Self::inner_new(&heap.device, raw, desc, access, state, Some(allocation))
    }
}

impl ImageResource for Image {
    fn get_barrier(
        &self,
        state: ResourceStates,
        subresource: Option<SubresourceIndex>,
    ) -> Option<dx::ResourceBarrier<'_>> {
        let index = if let Some(subindex) = subresource {
            assert!(!self.is_array() || subindex.array_index == 0);

            Some(subindex.mip_index + subindex.array_index * (self.desc.mip_levels as usize))
        } else {
            None
        };

        if let Some(index) = index {
            let old = self.state[index].swap(state, std::sync::atomic::Ordering::Relaxed);

            if old != state {
                Some(dx::ResourceBarrier::transition(
                    self.get_raw(),
                    old.as_raw(),
                    state.as_raw(),
                    Some(index),
                ))
            } else {
                None
            }
        } else {
            //TODO: Return smallvec?
            let old = self.state[0].swap(state, std::sync::atomic::Ordering::Relaxed);
            self.state
                .iter()
                .skip(1)
                .for_each(|s| s.store(state, std::sync::atomic::Ordering::Relaxed));

            if old != state {
                Some(dx::ResourceBarrier::transition(
                    self.get_raw(),
                    old.as_raw(),
                    state.as_raw(),
                    None,
                ))
            } else {
                None
            }
        }
    }
}

impl ShareableImage for Image {}

#[derive(Clone, Debug)]
pub struct ImageDesc {
    width: u32,
    height: u32,
    count: u8,
    mip_levels: u8,
    format: dx::Format,
    layout: dx::TextureLayout,
    usage: TextureUsage,
    flags: dx::ResourceFlags,
}

impl ImageDesc {
    pub fn new(width: u32, height: u32, format: dx::Format) -> Self {
        Self {
            width,
            height,
            count: 1,
            mip_levels: 1,
            format,
            layout: dx::TextureLayout::Unknown,
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
            TextureUsage::RenderTarget { srv, uav, .. } => {
                self.flags = dx::ResourceFlags::AllowRenderTarget;

                if !srv {
                    self.flags |= dx::ResourceFlags::DenyShaderResource;
                }

                if *uav {
                    self.flags |= dx::ResourceFlags::AllowUnorderedAccess;
                }
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

impl From<ImageDesc> for dx::ResourceDesc {
    fn from(val: ImageDesc) -> Self {
        dx::ResourceDesc::texture_2d(val.width, val.height)
            .with_array_size(val.count as u16)
            .with_format(val.format)
            .with_flags(val.flags)
            .with_layout(val.layout)
            .with_mip_levels(val.mip_levels as u16)
    }
}

impl ResourceDesc for ImageDesc {}
impl ImageResourceDesc for ImageDesc {
    fn clear_color(&self) -> Option<dx::ClearValue> {
        match &self.usage {
            TextureUsage::RenderTarget { color, .. } => {
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

impl ShareableImageDesc for ImageDesc {
    fn flags(&self) -> dx::ResourceFlags {
        self.flags
    }

    fn with_flags(mut self, flags: dx::ResourceFlags) -> Self {
        self.flags = flags;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageViewDesc<T: ViewType> {
    format: Option<dx::Format>,
    mip_base: u8,
    mip_slice: u8,
    array: Option<Range<u8>>,
    _marker: PhantomData<T>,
}

impl ImageViewDesc<RtvView> {
    pub fn rtv() -> Self {
        Self {
            format: None,
            mip_base: 1,
            mip_slice: 1,
            array: None,
            _marker: PhantomData,
        }
    }

    pub fn with_mip_base(mut self, mip_base: u8) -> Self {
        self.mip_base = mip_base;
        self
    }
}

impl ImageViewDesc<DsvView> {
    pub fn dsv() -> Self {
        Self {
            format: None,
            mip_base: 1,
            mip_slice: 1,
            array: None,
            _marker: PhantomData,
        }
    }
}

impl ImageViewDesc<SrvView> {
    pub fn srv() -> Self {
        Self {
            format: None,
            mip_base: 1,
            mip_slice: 1,
            array: None,
            _marker: PhantomData,
        }
    }
}

impl ImageViewDesc<UavView> {
    pub fn srv() -> Self {
        Self {
            format: None,
            mip_base: 1,
            mip_slice: 1,
            array: None,
            _marker: PhantomData,
        }
    }
}

impl<T: ViewType> ImageViewDesc<T> {
    pub fn with_format(mut self, format: dx::Format) -> Self {
        self.format = Some(format);
        self
    }

    pub fn with_mip_slice(mut self, mip_slice: u8) -> Self {
        self.mip_slice = mip_slice;
        self
    }

    pub fn with_array(mut self, array: Range<u8>) -> Self {
        self.array = Some(array);
        self
    }

    pub fn is_array(&self) -> bool {
        self.array.is_some()
    }
}

impl From<ImageViewDesc<RtvView>> for dx::RenderTargetViewDesc {
    fn from(value: ImageViewDesc<RtvView>) -> Self {
        if let Some(array) = value.array {
            Self::texture_2d_array(
                value.format.unwrap_or(dx::Format::Unknown),
                value.mip_slice as u32,
                0,
                Range {
                    start: array.start as u32,
                    end: array.end as u32,
                },
            )
        } else {
            Self::texture_2d(
                value.format.unwrap_or(dx::Format::Unknown),
                value.mip_slice as u32,
                0,
            )
        }
    }
}

impl From<ImageViewDesc<DsvView>> for dx::DepthStencilViewDesc {
    fn from(value: ImageViewDesc<DsvView>) -> Self {
        if let Some(array) = value.array {
            Self::texture_2d_array(
                value.format.unwrap_or(dx::Format::Unknown),
                value.mip_slice as u32,
                Range {
                    start: array.start as u32,
                    end: array.end as u32,
                },
            )
        } else {
            Self::texture_2d(
                value.format.unwrap_or(dx::Format::Unknown),
                value.mip_slice as u32,
            )
        }
    }
}

impl From<ImageViewDesc<SrvView>> for dx::ShaderResourceViewDesc {
    fn from(value: ImageViewDesc<SrvView>) -> Self {
        if let Some(array) = value.array {
            Self::texture_2d_array(
                value.format.unwrap_or(dx::Format::Unknown),
                value.mip_base as u32,
                value.mip_slice as u32,
                0.0,
                0,
                Range {
                    start: array.start as u32,
                    end: array.end as u32,
                },
            )
        } else {
            Self::texture_2d(
                value.format.unwrap_or(dx::Format::Unknown),
                value.mip_base as u32,
                value.mip_slice as u32,
                0.0,
                0,
            )
        }
    }
}

impl From<ImageViewDesc<UavView>> for dx::UnorderedAccessViewDesc {
    fn from(value: ImageViewDesc<UavView>) -> Self {
        if let Some(array) = value.array {
            Self::texture_2d_array(
                value.format.unwrap_or(dx::Format::Unknown),
                value.mip_slice as u32,
                0,
                Range {
                    start: array.start as u32,
                    end: array.end as u32,
                },
            )
        } else {
            Self::texture_2d(
                value.format.unwrap_or(dx::Format::Unknown),
                0,
                value.mip_slice as u32,
            )
        }
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::Image;

    const fn is_send_sync<T: Send + Sync>() {}

    const _: () = is_send_sync::<Image>();
}
