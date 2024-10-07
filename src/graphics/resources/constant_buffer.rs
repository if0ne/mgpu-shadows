use std::{
    fmt::Debug, marker::PhantomData, ops::Deref, ptr::NonNull, sync::Arc
};

use oxidx::dx::{self, IDevice, IResource};
use parking_lot::Mutex;

use crate::graphics::{
    descriptor_heap::{CbvView, DescriptorAllocator, ResourceDescriptor},
    device::Device,
    heaps::{Allocation, MemoryHeapType},
};

use super::{
    buffer::{BaseBuffer, Buffer},
    GpuAccess, Resource, ResourceDesc,
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
    size: usize,
    access: ConstantBufferGpuAccess,
    marker: PhantomData<T>,
}

impl<T: Clone> Buffer for ConstantBuffer<T> {}

impl<T: Clone> ConstantBuffer<T> {
    pub(in super::super) fn inner_new(
        resource: dx::Resource,
        desc: ConstantBufferDesc,
        access: GpuAccess,
        allocation: Option<Allocation>,
    ) -> Self {
        let mapped_data = resource.map::<T>(0, None).unwrap();

        let base_loc = resource.get_gpu_virtual_address();

        let access = match access {
            GpuAccess::Address => {
                ConstantBufferGpuAccess::Addresses(Self::create_addresses(base_loc, desc.size))
            }
            GpuAccess::Descriptor(descriptor_allocator) => ConstantBufferGpuAccess::Descriptors(
                Self::create_cbvs(base_loc, desc.size, &descriptor_allocator),
            ),
        };

        Self(Arc::new(ConstantBufferInner {
            buffer: BaseBuffer {
                raw: resource,
                size: desc.size * size_of::<T>(),
                state: Mutex::new(dx::ResourceStates::GenericRead),
                flags: desc.flags,
                allocation,
            },
            mapped_data: Mutex::new(mapped_data),
            size: desc.size,
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
            unsafe { std::slice::from_raw_parts::<T>(guard.as_ptr() as *const _, self.size) };
        slice[index].clone()
    }

    pub fn write(&self, index: usize, value: T) {
        let mut guard = self.mapped_data.lock();
        let slice =
            unsafe { std::slice::from_raw_parts_mut::<T>(guard.as_mut() as *mut _, self.size) };
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
            ConstantBufferGpuAccess::Descriptors(ref vec) => vec[index],
            _ => unreachable!(),
        }
    }
}

impl<T: Clone> Drop for ConstantBuffer<T> {
    fn drop(&mut self) {
        self.buffer.raw.unmap(0, None);
    }
}

impl<T: Clone> Resource for ConstantBuffer<T> {
    type Desc = ConstantBufferDesc;
    type Access = GpuAccess;

    fn get_raw(&self) -> &dx::Resource {
        &self.buffer.raw
    }

    fn set_current_state(&self, state: dx::ResourceStates) -> dx::ResourceStates {
        let mut guard = self.buffer.state.lock();
        let old = *guard;

        *guard = state;

        old
    }

    fn get_current_state(&self) -> dx::ResourceStates {
        *self.buffer.state.lock()
    }

    fn get_desc(&self) -> Self::Desc {
        ConstantBufferDesc {
            size: self.size,
            flags: self.buffer.flags,
        }
    }

    fn from_desc(
        device: &Device,
        desc: Self::Desc,
        access: Self::Access,
        _init_state: dx::ResourceStates,
        _clear_color: Option<&dx::ClearValue>,
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
                &dx::ResourceDesc::buffer(desc.size * element_byte_size),
                dx::ResourceStates::GenericRead,
                None,
            )
            .unwrap();

        Self::inner_new(resource, desc, access, None)
    }

    fn from_raw_placed(
        raw: dx::Resource,
        desc: Self::Desc,
        access: Self::Access,
        allocation: Allocation,
    ) -> Self {
        const {
            assert!(std::mem::align_of::<T>() == 256);
        };
        assert!(allocation.heap.mtype == MemoryHeapType::Cpu);

        Self::inner_new(raw, desc, access, Some(allocation))
    }
}

#[derive(Clone, Debug)]
pub struct ConstantBufferDesc {
    size: usize,
    flags: dx::ResourceFlags,
}

impl ConstantBufferDesc {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            flags: dx::ResourceFlags::empty(),
        }
    }
}

impl Into<dx::ResourceDesc> for ConstantBufferDesc {
    fn into(self) -> dx::ResourceDesc {
        dx::ResourceDesc::buffer(self.size)
    }
}

impl ResourceDesc for ConstantBufferDesc {
    fn flags(&self) -> dx::ResourceFlags {
        self.flags
    }

    fn with_flags(mut self, flags: dx::ResourceFlags) -> Self {
        self.flags = flags;
        self
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConstantBufferGpuAccess {
    Addresses(Vec<dx::GpuVirtualAddress>),
    Descriptors(Vec<ResourceDescriptor<CbvView>>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConstantBufferGpuItem {
    Address(dx::GpuVirtualAddress),
    Descriptor(ResourceDescriptor<CbvView>),
}
