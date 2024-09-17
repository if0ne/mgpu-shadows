#![allow(private_bounds)]

use std::marker::PhantomData;

use oxidx::dx::{self, IDescriptorHeap, IDevice};

#[derive(Clone, Copy, Debug)]
pub struct Descriptor<T: DescriptorHeapType>(usize, PhantomData<T>);

#[derive(Debug)]
pub struct DescriptorHeap<T: DescriptorHeapType> {
    device: dx::Device,
    inner: dx::DescriptorHeap,
    free_list: Vec<Descriptor<T>>,

    size: usize,
    capacity: usize,
    increment_size: usize,

    _marker: PhantomData<T>,
}

impl<T: DescriptorHeapType> DescriptorHeap<T> {
    fn inner_new(
        device: dx::Device,
        capacity: usize,
        desc: &dx::DescriptorHeapDesc,
        increment_size: usize,
    ) -> Self {
        let inner: dx::DescriptorHeap = device.create_descriptor_heap(desc).unwrap();

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

    pub fn remove(&mut self, handle: Descriptor<T>) {
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

    pub fn get(&mut self, handle: Descriptor<T>) -> dx::GpuDescriptorHandle {
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
    fn grow(&mut self) {
        let new_capacity = self.capacity * 2;
        let new_inner: dx::DescriptorHeap = self
            .device
            .create_descriptor_heap(&dx::DescriptorHeapDesc::rtv(new_capacity))
            .unwrap();
        self.device.copy_descriptors_simple(
            self.size as u32,
            new_inner.get_cpu_descriptor_handle_for_heap_start(),
            self.inner.get_cpu_descriptor_handle_for_heap_start(),
            dx::DescriptorHeapType::Rtv,
        );

        self.capacity = new_capacity;
        self.inner = new_inner;
    }

    pub fn rtv(device: dx::Device, chunk_size: usize) -> Self {
        let increment_size =
            device.get_descriptor_handle_increment_size(dx::DescriptorHeapType::Rtv);
        Self::inner_new(
            device,
            chunk_size,
            &dx::DescriptorHeapDesc::rtv(chunk_size),
            increment_size,
        )
    }

    pub fn push(
        &mut self,
        resource: &dx::Resource,
        desc: Option<&dx::RenderTargetViewDesc>,
    ) -> Descriptor<RtvHeapView> {
        if let Some(free) = self.free_list.pop() {
            self.size += 1;

            let handle = self
                .inner
                .get_cpu_descriptor_handle_for_heap_start()
                .advance(free.0, self.increment_size);
            self.device
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
            .create_render_target_view(Some(resource), desc, handle);

        let handle = Descriptor(self.size, PhantomData);
        self.size += 1;

        handle
    }
}

impl DescriptorHeap<DsvHeapView> {
    fn grow(&mut self) {
        let new_capacity = self.capacity * 2;
        let new_inner: dx::DescriptorHeap = self
            .device
            .create_descriptor_heap(&dx::DescriptorHeapDesc::dsv(new_capacity))
            .unwrap();
        self.device.copy_descriptors_simple(
            self.size as u32,
            new_inner.get_cpu_descriptor_handle_for_heap_start(),
            self.inner.get_cpu_descriptor_handle_for_heap_start(),
            dx::DescriptorHeapType::Dsv,
        );

        self.capacity = new_capacity;
        self.inner = new_inner;
    }

    pub fn dsv(device: dx::Device, chunk_size: usize) -> Self {
        let increment_size =
            device.get_descriptor_handle_increment_size(dx::DescriptorHeapType::Dsv);
        Self::inner_new(
            device,
            chunk_size,
            &dx::DescriptorHeapDesc::dsv(chunk_size),
            increment_size,
        )
    }

    pub fn push(
        &mut self,
        resource: &dx::Resource,
        desc: Option<&dx::DepthStencilViewDesc>,
    ) -> Descriptor<DsvHeapView> {
        if let Some(free) = self.free_list.pop() {
            self.size += 1;

            let handle = self
                .inner
                .get_cpu_descriptor_handle_for_heap_start()
                .advance(free.0, self.increment_size);
            self.device
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
            .create_depth_stencil_view(Some(resource), desc, handle);

        let handle = Descriptor(self.size, PhantomData);
        self.size += 1;

        handle
    }
}

impl DescriptorHeap<CbvSrvUavHeapView> {
    fn grow(&mut self) {
        let new_capacity = self.capacity * 2;
        let new_inner: dx::DescriptorHeap = self
            .device
            .create_descriptor_heap(&dx::DescriptorHeapDesc::cbr_srv_uav(new_capacity))
            .unwrap();
        self.device.copy_descriptors_simple(
            self.size as u32,
            new_inner.get_cpu_descriptor_handle_for_heap_start(),
            self.inner.get_cpu_descriptor_handle_for_heap_start(),
            dx::DescriptorHeapType::CbvSrvUav,
        );

        self.capacity = new_capacity;
        self.inner = new_inner;
    }

    pub fn cbr_srv_uav(device: dx::Device, chunk_size: usize) -> Self {
        let increment_size =
            device.get_descriptor_handle_increment_size(dx::DescriptorHeapType::CbvSrvUav);
        Self::inner_new(
            device,
            chunk_size,
            &dx::DescriptorHeapDesc::cbr_srv_uav(chunk_size)
                .with_flags(dx::DescriptorHeapFlags::ShaderVisible),
            increment_size,
        )
    }

    pub fn push_cbv(
        &mut self,
        desc: Option<&dx::ConstantBufferViewDesc>,
    ) -> Descriptor<CbvSrvUavHeapView> {
        if let Some(free) = self.free_list.pop() {
            self.size += 1;

            let handle = self
                .inner
                .get_cpu_descriptor_handle_for_heap_start()
                .advance(free.0, self.increment_size);
            self.device.create_constant_buffer_view(desc, handle);

            return free;
        }

        if self.size == self.capacity {
            self.grow();
        }

        let handle = self
            .inner
            .get_cpu_descriptor_handle_for_heap_start()
            .advance(self.size, self.increment_size);

        self.device.create_constant_buffer_view(desc, handle);

        let handle = Descriptor(self.size, PhantomData);
        self.size += 1;

        handle
    }

    pub fn push_srv(
        &mut self,
        resources: &dx::Resource,
        desc: Option<&dx::ShaderResourceViewDesc>,
    ) -> Descriptor<CbvSrvUavHeapView> {
        if let Some(free) = self.free_list.pop() {
            self.size += 1;

            let handle = self
                .inner
                .get_cpu_descriptor_handle_for_heap_start()
                .advance(free.0, self.increment_size);
            self.device
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
            .create_shader_resource_view(Some(resources), desc, handle);

        let handle = Descriptor(self.size, PhantomData);
        self.size += 1;

        handle
    }

    pub fn push_uav(
        &mut self,
        resources: &dx::Resource,
        counter_resources: Option<&dx::Resource>,
        desc: Option<&dx::UnorderedAccessViewDesc>,
    ) -> Descriptor<CbvSrvUavHeapView> {
        if let Some(free) = self.free_list.pop() {
            self.size += 1;

            let handle = self
                .inner
                .get_cpu_descriptor_handle_for_heap_start()
                .advance(free.0, self.increment_size);
            self.device.create_unordered_access_view(
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

        self.device
            .create_unordered_access_view(Some(resources), counter_resources, desc, handle);

        let handle = Descriptor(self.size, PhantomData);
        self.size += 1;

        handle
    }
}

trait DescriptorHeapType {}

struct RtvHeapView;
impl DescriptorHeapType for RtvHeapView {}

struct DsvHeapView;
impl DescriptorHeapType for DsvHeapView {}

struct CbvSrvUavHeapView;
impl DescriptorHeapType for CbvSrvUavHeapView {}
