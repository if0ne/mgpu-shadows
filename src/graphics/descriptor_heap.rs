#![allow(private_bounds)]

use std::marker::PhantomData;

use oxidx::dx::{self, IDescriptorHeap, IDevice};

use super::device::Device;

#[derive(Clone, Copy, Debug)]
pub struct ResourceDescriptor<T: DescriptorHeapType>(usize, PhantomData<T>);

#[derive(Debug)]
pub struct DescriptorHeap<T: DescriptorHeapType> {
    device: Device,
    inner: dx::DescriptorHeap,
    free_list: Vec<ResourceDescriptor<T>>,

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
            inner,
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
            self.inner.get_cpu_descriptor_handle_for_heap_start(),
            T::RAW_TYPE,
        );

        self.capacity = new_capacity;
        self.inner = new_inner;
    }

    pub fn remove(&mut self, handle: ResourceDescriptor<T>) {
        if handle.0 >= self.size {
            panic!(
                "HeapView<{}>: Index out of bounds, length {} and passed {}",
                std::any::type_name::<T>(),
                self.size,
                handle.0
            );
        }

        self.size -= 1;
        self.free_list.push(handle);
    }

    pub fn get(&mut self, handle: ResourceDescriptor<T>) -> dx::GpuDescriptorHandle {
        if handle.0 >= self.size {
            panic!(
                "DescriptorHeap<{}>: Index out of bounds, lenght {} and passed {}",
                std::any::type_name::<T>(),
                self.size,
                handle.0
            );
        }

        self.inner
            .get_gpu_descriptor_handle_for_heap_start()
            .advance(handle.0, self.increment_size)
    }
}

impl DescriptorHeap<RtvHeapView> {
    pub fn push(
        &mut self,
        resource: &dx::Resource,
        desc: Option<&dx::RenderTargetViewDesc>,
    ) -> ResourceDescriptor<RtvHeapView> {
        if let Some(free) = self.free_list.pop() {
            self.size += 1;

            let handle = self
                .inner
                .get_cpu_descriptor_handle_for_heap_start()
                .advance(free.0, self.increment_size);
            self.device
                .raw
                .create_render_target_view(Some(resource), desc, handle);

            return free;
        }

        if self.size == self.capacity {
            self.grow();
        }

        let handle = self
            .inner
            .get_cpu_descriptor_handle_for_heap_start()
            .advance(self.size, self.increment_size);

        self.device
            .raw
            .create_render_target_view(Some(resource), desc, handle);

        let handle = ResourceDescriptor(self.size, PhantomData);
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
        if let Some(free) = self.free_list.pop() {
            self.size += 1;

            let handle = self
                .inner
                .get_cpu_descriptor_handle_for_heap_start()
                .advance(free.0, self.increment_size);
            self.device
                .raw
                .create_depth_stencil_view(Some(resource), desc, handle);

            return free;
        }

        if self.size == self.capacity {
            self.grow();
        }

        let handle = self
            .inner
            .get_cpu_descriptor_handle_for_heap_start()
            .advance(self.size, self.increment_size);

        self.device
            .raw
            .create_depth_stencil_view(Some(resource), desc, handle);

        let handle = ResourceDescriptor(self.size, PhantomData);
        self.size += 1;

        handle
    }
}

impl DescriptorHeap<CbvSrvUavHeapView> {
    pub fn push_cbv(
        &mut self,
        desc: Option<&dx::ConstantBufferViewDesc>,
    ) -> ResourceDescriptor<CbvSrvUavHeapView> {
        if let Some(free) = self.free_list.pop() {
            self.size += 1;

            let handle = self
                .inner
                .get_cpu_descriptor_handle_for_heap_start()
                .advance(free.0, self.increment_size);
            self.device.raw.create_constant_buffer_view(desc, handle);

            return free;
        }

        if self.size == self.capacity {
            self.grow();
        }

        let handle = self
            .inner
            .get_cpu_descriptor_handle_for_heap_start()
            .advance(self.size, self.increment_size);

        self.device.raw.create_constant_buffer_view(desc, handle);

        let handle = ResourceDescriptor(self.size, PhantomData);
        self.size += 1;

        handle
    }

    pub fn push_srv(
        &mut self,
        resources: &dx::Resource,
        desc: Option<&dx::ShaderResourceViewDesc>,
    ) -> ResourceDescriptor<CbvSrvUavHeapView> {
        if let Some(free) = self.free_list.pop() {
            self.size += 1;

            let handle = self
                .inner
                .get_cpu_descriptor_handle_for_heap_start()
                .advance(free.0, self.increment_size);
            self.device
                .raw
                .create_shader_resource_view(Some(resources), desc, handle);

            return free;
        }

        if self.size == self.capacity {
            self.grow();
        }

        let handle = self
            .inner
            .get_cpu_descriptor_handle_for_heap_start()
            .advance(self.size, self.increment_size);

        self.device
            .raw
            .create_shader_resource_view(Some(resources), desc, handle);

        let handle = ResourceDescriptor(self.size, PhantomData);
        self.size += 1;

        handle
    }

    pub fn push_uav(
        &mut self,
        resources: &dx::Resource,
        counter_resources: Option<&dx::Resource>,
        desc: Option<&dx::UnorderedAccessViewDesc>,
    ) -> ResourceDescriptor<CbvSrvUavHeapView> {
        if let Some(free) = self.free_list.pop() {
            self.size += 1;

            let handle = self
                .inner
                .get_cpu_descriptor_handle_for_heap_start()
                .advance(free.0, self.increment_size);
            self.device.raw.create_unordered_access_view(
                Some(resources),
                counter_resources,
                desc,
                handle,
            );

            return free;
        }

        if self.size == self.capacity {
            self.grow();
        }

        let handle = self
            .inner
            .get_cpu_descriptor_handle_for_heap_start()
            .advance(self.size, self.increment_size);

        self.device.raw.create_unordered_access_view(
            Some(resources),
            counter_resources,
            desc,
            handle,
        );

        let handle = ResourceDescriptor(self.size, PhantomData);
        self.size += 1;

        handle
    }
}

pub(super) trait DescriptorHeapType {
    const RAW_TYPE: dx::DescriptorHeapType;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc;
}

pub(super) struct RtvHeapView;
impl DescriptorHeapType for RtvHeapView {
    const RAW_TYPE: dx::DescriptorHeapType = dx::DescriptorHeapType::Rtv;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc {
        dx::DescriptorHeapDesc::rtv(num)
    }
}

pub(super) struct DsvHeapView;
impl DescriptorHeapType for DsvHeapView {
    const RAW_TYPE: dx::DescriptorHeapType = dx::DescriptorHeapType::Dsv;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc {
        dx::DescriptorHeapDesc::dsv(num)
    }
}

pub(super) struct CbvSrvUavHeapView;
impl DescriptorHeapType for CbvSrvUavHeapView {
    const RAW_TYPE: dx::DescriptorHeapType = dx::DescriptorHeapType::CbvSrvUav;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc {
        dx::DescriptorHeapDesc::cbr_srv_uav(num)
    }
}
