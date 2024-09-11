#![allow(private_bounds)]

use std::marker::PhantomData;

use oxidx::dx::{self, IDescriptorHeap, IDevice};

#[derive(Clone, Copy, Debug)]
pub struct HeapViewHandle(usize);

#[derive(Debug)]
pub struct DescriptorHeap<T: DescriptorHeapType> {
    device: dx::Device,
    inner: Vec<dx::DescriptorHeap>,
    chunk_size: usize,
    allocated: usize,
    increment_size: usize,
    free_list: Vec<HeapViewHandle>,
    _marker: PhantomData<T>,
}

impl<T: DescriptorHeapType> DescriptorHeap<T> {
    fn inner_new(
        device: dx::Device,
        chunk_size: usize,
        desc: &dx::DescriptorHeapDesc,
        increment_size: usize,
    ) -> Self {
        let base: dx::DescriptorHeap = device.create_descriptor_heap(desc).unwrap();

        Self {
            device,
            inner: vec![base],
            chunk_size,
            allocated: 0,
            increment_size,
            free_list: vec![],
            _marker: PhantomData,
        }
    }

    pub fn remove(&mut self, handle: HeapViewHandle) {
        if handle.0 >= self.allocated {
            panic!(
                "HeapView<{}>: Index out of bounds, lenght {} and passed {}",
                std::any::type_name::<T>(),
                self.allocated,
                handle.0
            );
        }

        self.allocated -= 1;
        self.free_list.push(handle);
    }
}

impl DescriptorHeap<RtvHeapView> {
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
    ) -> HeapViewHandle {
        if let Some(free) = self.free_list.pop() {
            self.allocated += 1;
            return free;
        }

        if self.inner.len() <= self.allocated / self.chunk_size {
            let base: dx::DescriptorHeap = self
                .device
                .create_descriptor_heap(&dx::DescriptorHeapDesc::rtv(self.chunk_size))
                .unwrap();
            self.inner.push(base);
        }

        let handle = self.inner[self.allocated / self.chunk_size]
            .get_cpu_descriptor_handle_for_heap_start()
            .advance(self.allocated % self.chunk_size, self.increment_size);

        self.device
            .create_render_target_view(Some(resource), desc, handle);

        let handle = HeapViewHandle(self.allocated);
        self.allocated += 1;

        handle
    }
}

impl DescriptorHeap<DsvHeapView> {
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
    ) -> HeapViewHandle {
        if let Some(free) = self.free_list.pop() {
            self.allocated += 1;
            return free;
        }

        if self.inner.len() <= self.allocated / self.chunk_size {
            let base: dx::DescriptorHeap = self
                .device
                .create_descriptor_heap(&dx::DescriptorHeapDesc::dsv(self.chunk_size))
                .unwrap();
            self.inner.push(base);
        }

        let handle = self.inner[self.allocated / self.chunk_size]
            .get_cpu_descriptor_handle_for_heap_start()
            .advance(self.allocated % self.chunk_size, self.increment_size);

        self.device
            .create_depth_stencil_view(Some(resource), desc, handle);

        let handle = HeapViewHandle(self.allocated);
        self.allocated += 1;

        handle
    }
}

impl DescriptorHeap<CbvSrvUavHeapView> {
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

    pub fn push(
        &mut self,
        resource: &dx::Resource,
        desc: Option<&dx::DepthStencilViewDesc>,
    ) -> HeapViewHandle {
        if let Some(free) = self.free_list.pop() {
            self.allocated += 1;
            return free;
        }

        if self.inner.len() <= self.allocated / self.chunk_size {
            let base: dx::DescriptorHeap = self
                .device
                .create_descriptor_heap(&dx::DescriptorHeapDesc::cbr_srv_uav(self.chunk_size))
                .unwrap();
            self.inner.push(base);
        }

        let handle = self.inner[self.allocated / self.chunk_size]
            .get_cpu_descriptor_handle_for_heap_start()
            .advance(self.allocated % self.chunk_size, self.increment_size);

        self.device
            .create_depth_stencil_view(Some(resource), desc, handle);

        let handle = HeapViewHandle(self.allocated);
        self.allocated += 1;

        handle
    }

    pub fn get(&mut self, handle: HeapViewHandle) -> dx::GpuDescriptorHandle {
        if handle.0 >= self.allocated {
            panic!(
                "HeapView<{}>: Index out of bounds, lenght {} and passed {}",
                std::any::type_name::<CbvSrvUavHeapView>(),
                self.allocated,
                handle.0
            );
        }

        self.inner[handle.0 / self.chunk_size]
            .get_gpu_descriptor_handle_for_heap_start()
            .advance(handle.0 % self.chunk_size, self.increment_size)
    }
}

trait DescriptorHeapType {}

struct RtvHeapView;
impl DescriptorHeapType for RtvHeapView {}

struct DsvHeapView;
impl DescriptorHeapType for DsvHeapView {}

struct CbvSrvUavHeapView;
impl DescriptorHeapType for CbvSrvUavHeapView {}
