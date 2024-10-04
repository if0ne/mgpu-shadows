use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr::NonNull,
};

use oxidx::dx::{self, IDevice, IResource};

use crate::graphics::{
    device::Device,
    heaps::{Allocation, Upload},
};

use super::{
    buffer::{BaseBuffer, Buffer},
    Resource,
};

#[derive(Debug)]
pub struct ConstantBuffer<T: Clone> {
    buffer: BaseBuffer<Upload>,
    mapped_data: NonNull<T>,
    size: usize,
    marker: PhantomData<T>,
}

impl<T: Clone> Buffer for ConstantBuffer<T> {}

impl<T: Clone> ConstantBuffer<T> {
    pub(in super::super) fn inner_new(device: &Device, size: usize) -> Self {
        const {
            assert!(std::mem::align_of::<T>() == 256);
        };

        let element_byte_size = size_of::<T>();

        let resource: dx::Resource = device
            .raw
            .create_committed_resource(
                &dx::HeapProperties::upload(),
                dx::HeapFlags::empty(),
                &dx::ResourceDesc::buffer(size * element_byte_size),
                dx::ResourceStates::GenericRead,
                None,
            )
            .unwrap();

        let mapped_data = resource.map(0, None).unwrap();

        Self {
            buffer: BaseBuffer {
                raw: resource,
                state: dx::ResourceStates::GenericRead,
                allocation: None,
            },
            mapped_data,
            size,
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

    fn get_desc(&self) -> Self::Desc {
        todo!()
    }

    fn from_desc<H: crate::graphics::heaps::HeapType>(device: &Device, desc: Self::Desc) -> Self {
        todo!()
    }

    fn from_raw_placed<H: crate::graphics::heaps::HeapType>(
        raw: dx::Resource,
        desc: Self::Desc,
        allocation: Allocation<Upload>,
    ) -> Self {
        let mapped = raw.map(0, None).unwrap();
        Self {
            buffer: BaseBuffer {
                raw,
                state: dx::ResourceStates::GenericRead,
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
}
