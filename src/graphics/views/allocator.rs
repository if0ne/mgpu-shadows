use std::{marker::PhantomData, ops::Deref, sync::Arc};

use oxidx::dx;
use parking_lot::Mutex;

use crate::graphics::device::Device;

use super::{heap::ViewHeap, CbvSrvUavView, CbvView, DsvView, GpuView, RtvView, SamplerView, SrvView, UavView};

#[derive(Clone, Debug)]
pub struct ViewAllocator(Arc<DescriptorAllocatorInner>);

#[derive(Debug)]
pub struct DescriptorAllocatorInner {
    rtv: Mutex<ViewHeap<RtvView>>,
    dsv: Mutex<ViewHeap<DsvView>>,
    cbv_srv_uav: Mutex<ViewHeap<CbvSrvUavView>>,
    sampler: Mutex<ViewHeap<SamplerView>>,
}

impl ViewAllocator {
    pub(crate) fn inner_new(
        device: &Device,
        rtv_size: usize,
        dsv_size: usize,
        cbv_srv_uav_size: usize,
        sampler_size: usize,
    ) -> Self {
        Self(Arc::new(DescriptorAllocatorInner {
            rtv: Mutex::new(ViewHeap::inner_new(device.clone(), rtv_size)),
            dsv: Mutex::new(ViewHeap::inner_new(device.clone(), dsv_size)),
            cbv_srv_uav: Mutex::new(ViewHeap::inner_new(device.clone(), cbv_srv_uav_size)),
            sampler: Mutex::new(ViewHeap::inner_new(device.clone(), sampler_size)),
        }))
    }
}

impl ViewAllocator {
    pub fn remove_rtv(&self, handle: GpuView<RtvView>) {
        self.rtv.lock().remove(handle)
    }

    pub fn remove_dsv(&self, handle: GpuView<DsvView>) {
        self.dsv.lock().remove(handle)
    }

    pub fn remove_cbv(&self, handle: GpuView<CbvView>) {
        self.cbv_srv_uav.lock().remove(GpuView {
            index: handle.index,
            gpu: handle.gpu,
            cpu: handle.cpu,
            _marker: PhantomData,
        })
    }

    pub fn remove_srv(&self, handle: GpuView<SrvView>) {
        self.cbv_srv_uav.lock().remove(GpuView {
            index: handle.index,
            gpu: handle.gpu,
            cpu: handle.cpu,
            _marker: PhantomData,
        })
    }

    pub fn remove_uav(&self, handle: GpuView<UavView>) {
        self.cbv_srv_uav.lock().remove(GpuView {
            index: handle.index,
            gpu: handle.gpu,
            cpu: handle.cpu,
            _marker: PhantomData,
        })
    }

    pub fn remove_sampler(&self, handle: GpuView<SamplerView>) {
        self.sampler.lock().remove(handle)
    }

    pub fn push_rtv(
        &self,
        resource: &dx::Resource,
        desc: Option<&dx::RenderTargetViewDesc>,
    ) -> GpuView<RtvView> {
        self.rtv.lock().push(resource, desc)
    }

    pub fn push_dsv(
        &self,
        resource: &dx::Resource,
        desc: Option<&dx::DepthStencilViewDesc>,
    ) -> GpuView<DsvView> {
        self.dsv.lock().push(resource, desc)
    }

    pub fn push_sampler(&self, desc: &dx::SamplerDesc) -> GpuView<SamplerView> {
        self.sampler.lock().push(desc)
    }

    pub fn push_cbv(&self, desc: Option<&dx::ConstantBufferViewDesc>) -> GpuView<CbvView> {
        self.cbv_srv_uav.lock().push_cbv(desc)
    }

    pub fn push_srv(
        &self,
        resource: &dx::Resource,
        desc: Option<&dx::ShaderResourceViewDesc>,
    ) -> GpuView<SrvView> {
        self.cbv_srv_uav.lock().push_srv(resource, desc)
    }

    pub fn push_uav(
        &self,
        resource: &dx::Resource,
        counter_resource: Option<&dx::Resource>,
        desc: Option<&dx::UnorderedAccessViewDesc>,
    ) -> GpuView<UavView> {
        self.cbv_srv_uav
            .lock()
            .push_uav(resource, counter_resource, desc)
    }
}

impl Deref for ViewAllocator {
    type Target = DescriptorAllocatorInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
