use std::{fmt::Debug, marker::PhantomData, ops::Deref, ptr::NonNull, sync::Arc};

use atomig::Atomic;
use oxidx::dx::{self, IDevice, IResource};
use parking_lot::Mutex;

use crate::graphics::{
    descriptor_heap::{CbvView, DescriptorAllocator, ResourceDescriptor},
    device::Device,
    heaps::{Allocation, MemoryHeap, MemoryHeapType},
};

use super::{
    buffer::BaseBuffer, Buffer, BufferDesc, GpuAccess, Resource, ResourceDesc, ResourceStates,
};

#[derive(Clone, Debug)]
pub struct ConstantBuffer<T: Clone>(Arc<ConstantBufferInner<T>>);

impl<T: Clone> Deref for ConstantBuffer<T> {
    type Target = ConstantBufferInner<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct ConstantBufferInner<T: Clone> {
    buffer: BaseBuffer,
    mapped_data: Mutex<NonNull<T>>,
    count: usize,
    access: ConstantBufferGpuAccess,
    marker: PhantomData<T>,
}

impl<T: Clone> ConstantBuffer<T> {
    pub(in super::super) fn inner_new(
        resource: dx::Resource,
        desc: ConstantBufferDesc<T>,
        access: GpuAccess,
        state: ResourceStates,
        allocation: Option<Allocation>,
    ) -> Self {
        let mapped_data = resource.map::<T>(0, None).unwrap();

        let base_loc = resource.get_gpu_virtual_address();

        let access = match access {
            GpuAccess::Address => {
                ConstantBufferGpuAccess::Addresses(Self::create_addresses(base_loc, desc.count))
            }
            GpuAccess::Descriptor(descriptor_allocator) => ConstantBufferGpuAccess::Descriptors(
                descriptor_allocator.clone(),
                Self::create_cbvs(base_loc, desc.count, &descriptor_allocator),
            ),
        };

        Self(Arc::new(ConstantBufferInner {
            buffer: BaseBuffer {
                raw: resource,
                size: desc.count * size_of::<T>(),
                state: Atomic::new(state),
                flags: dx::ResourceFlags::empty(),
                allocation,
            },
            mapped_data: Mutex::new(mapped_data),
            count: desc.count,
            access,
            marker: PhantomData,
        }))
    }

    fn create_cbvs(
        base_loc: dx::GpuVirtualAddress,
        size: usize,
        allocator: &DescriptorAllocator,
    ) -> Vec<ResourceDescriptor<CbvView>> {
        (0..size)
            .map(|i| {
                let offset = base_loc + (i * size_of::<T>()) as u64;
                allocator.push_cbv(Some(&dx::ConstantBufferViewDesc::new(
                    offset,
                    size_of::<T>() as u32,
                )))
            })
            .collect()
    }

    fn create_addresses(
        base_loc: dx::GpuVirtualAddress,
        size: usize,
    ) -> Vec<dx::GpuVirtualAddress> {
        (0..size)
            .map(|i| base_loc + (i * size_of::<T>()) as u64)
            .collect()
    }
}

impl<T: Clone> ConstantBuffer<T> {
    pub fn read(&self, index: usize) -> T {
        let guard = self.mapped_data.lock();
        let slice =
            unsafe { std::slice::from_raw_parts::<T>(guard.as_ptr() as *const _, self.count) };
        slice[index].clone()
    }

    pub fn write(&self, index: usize, value: T) {
        let mut guard = self.mapped_data.lock();
        let slice =
            unsafe { std::slice::from_raw_parts_mut::<T>(guard.as_mut() as *mut _, self.count) };
        slice[index] = value;
    }

    pub fn get_address(&self, index: usize) -> dx::GpuVirtualAddress {
        match self.access {
            ConstantBufferGpuAccess::Addresses(ref vec) => vec[index],
            _ => unreachable!(),
        }
    }

    pub fn get_descriptor(&self, index: usize) -> ResourceDescriptor<CbvView> {
        match self.access {
            ConstantBufferGpuAccess::Descriptors(_, ref vec) => vec[index],
            _ => unreachable!(),
        }
    }
}

impl<T: Clone> Drop for ConstantBufferInner<T> {
    fn drop(&mut self) {
        self.buffer.raw.unmap(0, None);

        if let ConstantBufferGpuAccess::Descriptors(ref allocator, ref mut vec) = self.access {
            let handles = std::mem::take(vec);

            for handle in handles {
                allocator.remove_cbv(handle);
            }
        }
    }
}

impl<T: Clone> Resource for ConstantBuffer<T> {
    type Desc = ConstantBufferDesc<T>;
    type Access = GpuAccess;

    fn get_raw(&self) -> &dx::Resource {
        &self.buffer.raw
    }

    fn get_barrier(
        &self,
        _state: ResourceStates,
        _subresource: usize,
    ) -> Option<dx::ResourceBarrier<'_>> {
        None
    }

    fn get_desc(&self) -> Self::Desc {
        ConstantBufferDesc {
            count: self.count,
            _marker: PhantomData,
        }
    }

    fn from_desc(
        device: &Device,
        desc: Self::Desc,
        access: Self::Access,
        _init_state: ResourceStates,
    ) -> Self {
        const {
            assert!(std::mem::align_of::<T>() == 256);
        };
        let element_byte_size = size_of::<T>();

        let resource: dx::Resource = device
            .raw
            .create_committed_resource(
                &dx::HeapProperties::upload(),
                dx::HeapFlags::empty(),
                &dx::ResourceDesc::buffer(desc.count * element_byte_size),
                dx::ResourceStates::GenericRead,
                None,
            )
            .unwrap();

        Self::inner_new(resource, desc, access, ResourceStates::GenericRead, None)
    }

    fn from_raw_placed(
        _heap: &MemoryHeap,
        raw: dx::Resource,
        desc: Self::Desc,
        access: Self::Access,
        _state: ResourceStates,
        allocation: Allocation,
    ) -> Self {
        const {
            assert!(std::mem::align_of::<T>() == 256);
        };
        assert!(allocation.heap.mtype == MemoryHeapType::Cpu);

        Self::inner_new(
            raw,
            desc,
            access,
            ResourceStates::GenericRead,
            Some(allocation),
        )
    }
}

impl<T: Clone> Buffer for ConstantBuffer<T> {}

#[derive(Clone, Debug)]
pub struct ConstantBufferDesc<T> {
    count: usize,
    _marker: PhantomData<T>,
}

impl<T> ConstantBufferDesc<T> {
    pub fn new(count: usize) -> Self {
        Self {
            count,
            _marker: PhantomData,
        }
    }
}

impl<T> Into<dx::ResourceDesc> for ConstantBufferDesc<T> {
    fn into(self) -> dx::ResourceDesc {
        dx::ResourceDesc::buffer(self.count * size_of::<T>())
    }
}

impl<T: Clone> ResourceDesc for ConstantBufferDesc<T> {}
impl<T: Clone> BufferDesc for ConstantBufferDesc<T> {}

#[derive(Debug)]
pub enum ConstantBufferGpuAccess {
    Addresses(Vec<dx::GpuVirtualAddress>),
    Descriptors(DescriptorAllocator, Vec<ResourceDescriptor<CbvView>>),
}
