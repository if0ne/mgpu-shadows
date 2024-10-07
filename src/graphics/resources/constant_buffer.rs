use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr::NonNull,
};

use oxidx::dx::{self, IDevice, IResource};
use parking_lot::Mutex;

use crate::graphics::{
    device::Device,
    heaps::{Allocation, MemoryHeapType},
};

use super::{
    buffer::{BaseBuffer, Buffer},
    Resource, ResourceDesc,
};

#[derive(Debug)]
pub struct ConstantBuffer<T: Clone> {
    buffer: BaseBuffer,
    mapped_data: NonNull<T>,
    size: usize,
    marker: PhantomData<T>,
}

impl<T: Clone> Buffer for ConstantBuffer<T> {}

impl<T: Clone> ConstantBuffer<T> {
    pub(in super::super) fn inner_new(device: &Device, desc: ConstantBufferDesc) -> Self {
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

        let mapped_data = resource.map(0, None).unwrap();

        Self {
            buffer: BaseBuffer {
                raw: resource,
                size: desc.size * element_byte_size,
                state: Mutex::new(dx::ResourceStates::GenericRead),
                flags: desc.flags,
                allocation: None,
            },
            mapped_data,
            size: desc.size,
            marker: PhantomData,
        }
    }

    fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.mapped_data.as_ptr() as *const _, self.size) }
    }

    fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.mapped_data.as_ptr(), self.size) }
    }
}

impl<T: Clone + Debug> Index<usize> for ConstantBuffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<T: Clone + Debug> IndexMut<usize> for ConstantBuffer<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_slice_mut()[index]
    }
}

impl<T: Clone> Drop for ConstantBuffer<T> {
    fn drop(&mut self) {
        self.buffer.raw.unmap(0, None);
    }
}

impl<T: Clone> Resource for ConstantBuffer<T> {
    type Desc = ConstantBufferDesc;

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
        _init_state: dx::ResourceStates,
        _clear_color: Option<&dx::ClearValue>,
    ) -> Self {
        Self::inner_new(device, desc)
    }

    fn from_raw_placed(raw: dx::Resource, desc: Self::Desc, allocation: Allocation) -> Self {
        const {
            assert!(std::mem::align_of::<T>() == 256);
        };

        assert!(allocation.heap.mtype == MemoryHeapType::Cpu);

        let element_byte_size = size_of::<T>();

        let mapped = raw.map(0, None).unwrap();
        Self {
            buffer: BaseBuffer {
                raw,
                state: Mutex::new(dx::ResourceStates::GenericRead),
                flags: desc.flags,
                size: desc.size * element_byte_size,
                allocation: Some(allocation),
            },
            mapped_data: mapped,
            size: desc.size,
            marker: PhantomData,
        }
    }
}

#[derive(Clone, Copy, Debug)]
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
