use std::{marker::PhantomData, ops::Deref, sync::Arc};

use oxidx::dx::{self, IDescriptorHeap, IDevice};
use parking_lot::Mutex;

use super::device::Device;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceDescriptor<T: DescriptorHeapType> {
    index: usize,
    gpu: dx::GpuDescriptorHandle,
    cpu: dx::CpuDescriptorHandle,
    _marker: PhantomData<T>,
}

impl<T: DescriptorHeapType> ResourceDescriptor<T> {
    pub fn gpu(&self) -> dx::GpuDescriptorHandle {
        self.gpu
    }

    pub fn cpu(&self) -> dx::CpuDescriptorHandle {
        self.cpu
    }
}

#[derive(Clone, Debug)]
pub struct DescriptorAllocator(Arc<DescriptorAllocatorInner>);

impl DescriptorAllocator {
    pub(super) fn inner_new(
        device: &Device,
        rtv_size: usize,
        dsv_size: usize,
        cbv_srv_uav_size: usize,
        sampler_size: usize,
    ) -> Self {
        Self(Arc::new(DescriptorAllocatorInner {
            rtv: Mutex::new(DescriptorHeap::inner_new(device.clone(), rtv_size)),
            dsv: Mutex::new(DescriptorHeap::inner_new(device.clone(), dsv_size)),
            cbv_srv_uav: Mutex::new(DescriptorHeap::inner_new(device.clone(), cbv_srv_uav_size)),
            sampler: Mutex::new(DescriptorHeap::inner_new(device.clone(), sampler_size)),
        }))
    }

    pub fn remove_rtv(&self, handle: ResourceDescriptor<RtvHeapView>) {
        self.rtv.lock().remove(handle)
    }

    pub fn remove_dsv(&self, handle: ResourceDescriptor<DsvHeapView>) {
        self.dsv.lock().remove(handle)
    }

    pub fn remove_cbv(&self, handle: ResourceDescriptor<CbvView>) {
        self.cbv_srv_uav.lock().remove(ResourceDescriptor {
            index: handle.index,
            gpu: handle.gpu,
            cpu: handle.cpu,
            _marker: PhantomData,
        })
    }

    pub fn remove_srv(&self, handle: ResourceDescriptor<SrvView>) {
        self.cbv_srv_uav.lock().remove(ResourceDescriptor {
            index: handle.index,
            gpu: handle.gpu,
            cpu: handle.cpu,
            _marker: PhantomData,
        })
    }

    pub fn remove_uav(&self, handle: ResourceDescriptor<UavView>) {
        self.cbv_srv_uav.lock().remove(ResourceDescriptor {
            index: handle.index,
            gpu: handle.gpu,
            cpu: handle.cpu,
            _marker: PhantomData,
        })
    }

    pub fn remove_sampler(&self, handle: ResourceDescriptor<SamplerView>) {
        self.sampler.lock().remove(handle)
    }

    pub fn push_rtv(
        &self,
        resource: &dx::Resource,
        desc: Option<&dx::RenderTargetViewDesc>,
    ) -> ResourceDescriptor<RtvHeapView> {
        self.rtv.lock().push(resource, desc)
    }

    pub fn push_dsv(
        &self,
        resource: &dx::Resource,
        desc: Option<&dx::DepthStencilViewDesc>,
    ) -> ResourceDescriptor<DsvHeapView> {
        self.dsv.lock().push(resource, desc)
    }

    pub fn push_sampler(&self, desc: &dx::SamplerDesc) -> ResourceDescriptor<SamplerView> {
        self.sampler.lock().push(desc)
    }

    pub fn push_cbv(
        &self,
        desc: Option<&dx::ConstantBufferViewDesc>,
    ) -> ResourceDescriptor<CbvView> {
        self.cbv_srv_uav.lock().push_cbv(desc)
    }

    pub fn push_srv(
        &self,
        resource: &dx::Resource,
        desc: Option<&dx::ShaderResourceViewDesc>,
    ) -> ResourceDescriptor<SrvView> {
        self.cbv_srv_uav.lock().push_srv(resource, desc)
    }

    pub fn push_uav(
        &self,
        resource: &dx::Resource,
        counter_resource: Option<&dx::Resource>,
        desc: Option<&dx::UnorderedAccessViewDesc>,
    ) -> ResourceDescriptor<UavView> {
        self.cbv_srv_uav
            .lock()
            .push_uav(resource, counter_resource, desc)
    }
}

impl Deref for DescriptorAllocator {
    type Target = DescriptorAllocatorInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct DescriptorAllocatorInner {
    rtv: Mutex<DescriptorHeap<RtvHeapView>>,
    dsv: Mutex<DescriptorHeap<DsvHeapView>>,
    cbv_srv_uav: Mutex<DescriptorHeap<CbvSrvUavHeapView>>,
    sampler: Mutex<DescriptorHeap<SamplerView>>,
}

#[derive(Debug)]
pub struct DescriptorHeap<T: DescriptorHeapType> {
    device: Device,
    raw: dx::DescriptorHeap,
    free_list: Vec<usize>,

    size: usize,
    capacity: usize,
    increment_size: usize,

    _marker: PhantomData<T>,
}

impl<T: DescriptorHeapType> DescriptorHeap<T> {
    pub(super) fn inner_new(device: Device, capacity: usize) -> Self {
        let inner: dx::DescriptorHeap = device
            .raw
            .create_descriptor_heap(&T::get_desc(capacity))
            .unwrap();
        let increment_size = device.raw.get_descriptor_handle_increment_size(T::RAW_TYPE);

        Self {
            device,
            raw: inner,
            free_list: vec![],

            increment_size,
            size: 0,
            capacity,

            _marker: PhantomData,
        }
    }

    fn grow(&mut self) {
        let new_capacity = self.capacity * 2;
        let new_inner: dx::DescriptorHeap = self
            .device
            .raw
            .create_descriptor_heap(&T::get_desc(new_capacity))
            .unwrap();
        self.device.raw.copy_descriptors_simple(
            self.size as u32,
            new_inner.get_cpu_descriptor_handle_for_heap_start(),
            self.raw.get_cpu_descriptor_handle_for_heap_start(),
            T::RAW_TYPE,
        );

        self.capacity = new_capacity;
        self.raw = new_inner;
    }

    pub fn remove(&mut self, handle: ResourceDescriptor<T>) {
        if handle.index >= self.size {
            panic!(
                "HeapView<{}>: Index out of bounds, length {} and passed {}",
                std::any::type_name::<T>(),
                self.size,
                handle.index
            );
        }

        self.size -= 1;
        self.free_list.push(handle.index);
    }
}

impl DescriptorHeap<RtvHeapView> {
    pub fn push(
        &mut self,
        resource: &dx::Resource,
        desc: Option<&dx::RenderTargetViewDesc>,
    ) -> ResourceDescriptor<RtvHeapView> {
        let index = if let Some(free) = self.free_list.pop() {
            free
        } else {
            if self.size == self.capacity {
                self.grow();
            }

            self.size
        };

        let handle = ResourceDescriptor {
            index,
            gpu: self
                .raw
                .get_gpu_descriptor_handle_for_heap_start()
                .advance(index, self.increment_size),
            cpu: self
                .raw
                .get_cpu_descriptor_handle_for_heap_start()
                .advance(index, self.increment_size),
            _marker: PhantomData,
        };

        self.device
            .raw
            .create_render_target_view(Some(resource), desc, handle.cpu());

        self.size += 1;

        handle
    }
}

impl DescriptorHeap<DsvHeapView> {
    pub fn push(
        &mut self,
        resource: &dx::Resource,
        desc: Option<&dx::DepthStencilViewDesc>,
    ) -> ResourceDescriptor<DsvHeapView> {
        let index = if let Some(free) = self.free_list.pop() {
            free
        } else {
            if self.size == self.capacity {
                self.grow();
            }

            self.size
        };

        let handle = ResourceDescriptor {
            index,
            gpu: self
                .raw
                .get_gpu_descriptor_handle_for_heap_start()
                .advance(index, self.increment_size),
            cpu: self
                .raw
                .get_cpu_descriptor_handle_for_heap_start()
                .advance(index, self.increment_size),
            _marker: PhantomData,
        };

        self.device
            .raw
            .create_depth_stencil_view(Some(resource), desc, handle.cpu());

        self.size += 1;

        handle
    }
}

impl DescriptorHeap<CbvSrvUavHeapView> {
    pub fn push_cbv(
        &mut self,
        desc: Option<&dx::ConstantBufferViewDesc>,
    ) -> ResourceDescriptor<CbvView> {
        let index = if let Some(free) = self.free_list.pop() {
            free
        } else {
            if self.size == self.capacity {
                self.grow();
            }

            self.size
        };

        let handle = ResourceDescriptor {
            index,
            gpu: self
                .raw
                .get_gpu_descriptor_handle_for_heap_start()
                .advance(index, self.increment_size),
            cpu: self
                .raw
                .get_cpu_descriptor_handle_for_heap_start()
                .advance(index, self.increment_size),
            _marker: PhantomData,
        };

        self.device
            .raw
            .create_constant_buffer_view(desc, handle.cpu());

        self.size += 1;

        handle
    }

    pub fn push_srv(
        &mut self,
        resources: &dx::Resource,
        desc: Option<&dx::ShaderResourceViewDesc>,
    ) -> ResourceDescriptor<SrvView> {
        let index = if let Some(free) = self.free_list.pop() {
            free
        } else {
            if self.size == self.capacity {
                self.grow();
            }

            self.size
        };

        let handle = ResourceDescriptor {
            index,
            gpu: self
                .raw
                .get_gpu_descriptor_handle_for_heap_start()
                .advance(index, self.increment_size),
            cpu: self
                .raw
                .get_cpu_descriptor_handle_for_heap_start()
                .advance(index, self.increment_size),
            _marker: PhantomData,
        };

        self.device
            .raw
            .create_shader_resource_view(Some(resources), desc, handle.cpu());

        self.size += 1;

        handle
    }

    pub fn push_uav(
        &mut self,
        resources: &dx::Resource,
        counter_resources: Option<&dx::Resource>,
        desc: Option<&dx::UnorderedAccessViewDesc>,
    ) -> ResourceDescriptor<UavView> {
        let index = if let Some(free) = self.free_list.pop() {
            free
        } else {
            if self.size == self.capacity {
                self.grow();
            }

            self.size
        };

        let handle = ResourceDescriptor {
            index,
            gpu: self
                .raw
                .get_gpu_descriptor_handle_for_heap_start()
                .advance(index, self.increment_size),
            cpu: self
                .raw
                .get_cpu_descriptor_handle_for_heap_start()
                .advance(index, self.increment_size),
            _marker: PhantomData,
        };

        self.device.raw.create_unordered_access_view(
            Some(resources),
            counter_resources,
            desc,
            handle.cpu(),
        );

        self.size += 1;

        handle
    }
}

impl DescriptorHeap<SamplerView> {
    pub fn push(&mut self, desc: &dx::SamplerDesc) -> ResourceDescriptor<SamplerView> {
        let index = if let Some(free) = self.free_list.pop() {
            free
        } else {
            if self.size == self.capacity {
                self.grow();
            }

            self.size
        };

        let handle = ResourceDescriptor {
            index,
            gpu: self
                .raw
                .get_gpu_descriptor_handle_for_heap_start()
                .advance(index, self.increment_size),
            cpu: self
                .raw
                .get_cpu_descriptor_handle_for_heap_start()
                .advance(index, self.increment_size),
            _marker: PhantomData,
        };

        self.device.raw.create_sampler(desc, handle.cpu());

        self.size += 1;

        handle
    }
}

pub(super) trait DescriptorHeapType {
    const RAW_TYPE: dx::DescriptorHeapType;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RtvHeapView;
impl DescriptorHeapType for RtvHeapView {
    const RAW_TYPE: dx::DescriptorHeapType = dx::DescriptorHeapType::Rtv;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc {
        dx::DescriptorHeapDesc::rtv(num)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DsvHeapView;
impl DescriptorHeapType for DsvHeapView {
    const RAW_TYPE: dx::DescriptorHeapType = dx::DescriptorHeapType::Dsv;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc {
        dx::DescriptorHeapDesc::dsv(num)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CbvSrvUavHeapView;
impl DescriptorHeapType for CbvSrvUavHeapView {
    const RAW_TYPE: dx::DescriptorHeapType = dx::DescriptorHeapType::CbvSrvUav;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc {
        dx::DescriptorHeapDesc::cbr_srv_uav(num)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CbvView;
impl DescriptorHeapType for CbvView {
    const RAW_TYPE: dx::DescriptorHeapType = dx::DescriptorHeapType::CbvSrvUav;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc {
        dx::DescriptorHeapDesc::cbr_srv_uav(num)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SrvView;
impl DescriptorHeapType for SrvView {
    const RAW_TYPE: dx::DescriptorHeapType = dx::DescriptorHeapType::CbvSrvUav;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc {
        dx::DescriptorHeapDesc::cbr_srv_uav(num)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UavView;
impl DescriptorHeapType for UavView {
    const RAW_TYPE: dx::DescriptorHeapType = dx::DescriptorHeapType::CbvSrvUav;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc {
        dx::DescriptorHeapDesc::cbr_srv_uav(num)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SamplerView;
impl DescriptorHeapType for SamplerView {
    const RAW_TYPE: dx::DescriptorHeapType = dx::DescriptorHeapType::Sampler;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc {
        dx::DescriptorHeapDesc::sampler(num)
    }
}
