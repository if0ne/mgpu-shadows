#![allow(private_bounds)]

use std::marker::PhantomData;

use oxidx::dx::{
    DepthStencilViewDesc, DescriptorHeap, DescriptorHeapDesc, DescriptorHeapFlags,
    DescriptorHeapType, Device, GpuDescriptorHandle, IDescriptorHeap, IDevice,
    RenderTargetViewDesc, Resource,
};

#[derive(Clone, Copy, Debug)]
pub struct HeapViewHandle(usize);

#[derive(Debug)]
pub struct HeapView<T: HeapViewType> {
    device: Device,
    inner: Vec<DescriptorHeap>,
    chunk_size: usize,
    allocated: usize,
    increment_size: usize,
    free_list: Vec<HeapViewHandle>,
    _marker: PhantomData<T>,
}

impl<T: HeapViewType> HeapView<T> {
    fn inner_new(
        device: Device,
        chunk_size: usize,
        desc: &DescriptorHeapDesc,
        increment_size: usize,
    ) -> Self {
        let base: DescriptorHeap = device.create_descriptor_heap(desc).unwrap();

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

impl HeapView<RtvHeapView> {
    pub fn rtv(device: Device, chunk_size: usize) -> Self {
        let increment_size = device.get_descriptor_handle_increment_size(DescriptorHeapType::Rtv);
        Self::inner_new(
            device,
            chunk_size,
            &DescriptorHeapDesc::rtv(chunk_size),
            increment_size,
        )
    }

    pub fn push(
        &mut self,
        resource: &Resource,
        desc: Option<&RenderTargetViewDesc>,
    ) -> HeapViewHandle {
        if let Some(free) = self.free_list.pop() {
            self.allocated += 1;
            return free;
        }

        if self.inner.len() <= self.allocated / self.chunk_size {
            let base: DescriptorHeap = self
                .device
                .create_descriptor_heap(&DescriptorHeapDesc::rtv(self.chunk_size))
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

impl HeapView<DsvHeapView> {
    pub fn dsv(device: Device, chunk_size: usize) -> Self {
        let increment_size = device.get_descriptor_handle_increment_size(DescriptorHeapType::Dsv);
        Self::inner_new(
            device,
            chunk_size,
            &DescriptorHeapDesc::dsv(chunk_size),
            increment_size,
        )
    }

    pub fn push(
        &mut self,
        resource: &Resource,
        desc: Option<&DepthStencilViewDesc>,
    ) -> HeapViewHandle {
        if let Some(free) = self.free_list.pop() {
            self.allocated += 1;
            return free;
        }

        if self.inner.len() <= self.allocated / self.chunk_size {
            let base: DescriptorHeap = self
                .device
                .create_descriptor_heap(&DescriptorHeapDesc::dsv(self.chunk_size))
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

impl HeapView<CbvSrvUavHeapView> {
    pub fn cbr_srv_uav(device: Device, chunk_size: usize) -> Self {
        let increment_size =
            device.get_descriptor_handle_increment_size(DescriptorHeapType::CbvSrvUav);
        Self::inner_new(
            device,
            chunk_size,
            &DescriptorHeapDesc::cbr_srv_uav(chunk_size)
                .with_flags(DescriptorHeapFlags::ShaderVisible),
            increment_size,
        )
    }

    pub fn push(
        &mut self,
        resource: &Resource,
        desc: Option<&DepthStencilViewDesc>,
    ) -> HeapViewHandle {
        if let Some(free) = self.free_list.pop() {
            self.allocated += 1;
            return free;
        }

        if self.inner.len() <= self.allocated / self.chunk_size {
            let base: DescriptorHeap = self
                .device
                .create_descriptor_heap(&DescriptorHeapDesc::cbr_srv_uav(self.chunk_size))
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

    pub fn get(&mut self, handle: HeapViewHandle) -> GpuDescriptorHandle {
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

trait HeapViewType {}

struct RtvHeapView;
impl HeapViewType for RtvHeapView {}

struct DsvHeapView;
impl HeapViewType for DsvHeapView {}

struct CbvSrvUavHeapView;
impl HeapViewType for CbvSrvUavHeapView {}
