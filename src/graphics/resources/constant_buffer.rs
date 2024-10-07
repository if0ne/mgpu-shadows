use std::{fmt::Debug, marker::PhantomData, ops::Deref, ptr::NonNull, sync::Arc};

use oxidx::dx::{self, IDevice, IResource};
use parking_lot::Mutex;

use crate::graphics::{
    descriptor_heap::{CbvView, DescriptorAllocator, ResourceDescriptor},
    device::Device,
    heaps::{Allocation, MemoryHeap, MemoryHeapType},
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
    count: usize,
    access: ConstantBufferGpuAccess,
    marker: PhantomData<T>,
}

impl<T: Clone> Buffer for ConstantBuffer<T> {}

impl<T: Clone> ConstantBuffer<T> {
    pub(in super::super) fn inner_new(
        resource: dx::Resource,
        desc: ConstantBufferDesc<T>,
        access: GpuAccess,
        state: dx::ResourceStates,
        allocation: Option<Allocation>,
    ) -> Self {
        let mapped_data = resource.map::<T>(0, None).unwrap();

        let base_loc = resource.get_gpu_virtual_address();

        let access = match access {
            GpuAccess::Address => {
                ConstantBufferGpuAccess::Addresses(Self::create_addresses(base_loc, desc.count))
            }
            GpuAccess::Descriptor(descriptor_allocator) => ConstantBufferGpuAccess::Descriptors(
                Self::create_cbvs(base_loc, desc.count, &descriptor_allocator),
            ),
        };

        Self(Arc::new(ConstantBufferInner {
            buffer: BaseBuffer {
                raw: resource,
                size: desc.count * size_of::<T>(),
                state: Mutex::new(state),
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
    type Desc = ConstantBufferDesc<T>;
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
            count: self.count,
            _marker: PhantomData,
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
                &dx::ResourceDesc::buffer(desc.count * element_byte_size),
                dx::ResourceStates::GenericRead,
                None,
            )
            .unwrap();

        Self::inner_new(
            resource,
            desc,
            access,
            dx::ResourceStates::GenericRead,
            None,
        )
    }

    fn from_raw_placed(
        _heap: &MemoryHeap,
        raw: dx::Resource,
        desc: Self::Desc,
        access: Self::Access,
        state: dx::ResourceStates,
        allocation: Allocation,
    ) -> Self {
        const {
            assert!(std::mem::align_of::<T>() == 256);
        };
        assert!(allocation.heap.mtype == MemoryHeapType::Cpu);

        Self::inner_new(raw, desc, access, state, Some(allocation))
    }
}

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

impl<T: Clone> ResourceDesc for ConstantBufferDesc<T> {
    fn flags(&self) -> dx::ResourceFlags {
        dx::ResourceFlags::empty()
    }

    fn with_flags(self, _flags: dx::ResourceFlags) -> Self {
        self
    }

    fn with_layout(self, _layout: dx::TextureLayout) -> Self {
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
