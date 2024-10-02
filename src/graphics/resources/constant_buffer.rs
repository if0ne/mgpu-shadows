use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr::NonNull,
};

use oxidx::dx::{self, IDevice, IResource};

use crate::graphics::{device::Device, heaps::Upload};

use super::buffer::{BaseBuffer, Buffer};

#[derive(Debug)]
pub struct ConstantBuffer<T: Clone> {
    buffer: BaseBuffer<Upload>,
    mapped_data: NonNull<ConstantDataWrapper<T>>,
    size: usize,
    marker: PhantomData<T>,
}

impl<T: Clone> Buffer for ConstantBufferType<T> {}

impl<T: Clone> ConstantBuffer<T> {
    pub(in super::super) fn inner_new(device: &Device, size: usize) -> Self {
        let element_byte_size = size_of::<ConstantDataWrapper<T>>();

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

    fn as_slice(&self) -> &[ConstantDataWrapper<T>] {
        unsafe {
            std::slice::from_raw_parts(self.inner.mapped_data.as_ptr() as *const _, self.inner.size)
        }
    }

    fn as_slice_mut(&mut self) -> &mut [ConstantDataWrapper<T>] {
        unsafe { std::slice::from_raw_parts_mut(self.inner.mapped_data.as_ptr(), self.inner.size) }
    }
}

impl<T: Clone + Debug> Index<usize> for ConstantBuffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &*self.as_slice()[index]
    }
}

impl<T: Clone + Debug> IndexMut<usize> for ConstantBuffer<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut *self.as_slice_mut()[index]
    }
}

impl<T: Clone> Drop for ConstantBuffer<T> {
    fn drop(&mut self) {
        self.buffer.raw.unmap(0, None);
    }
}

#[derive(Clone, Debug)]
#[repr(align(256))]
struct ConstantDataWrapper<T>(pub T);

impl<T: Clone> std::ops::Deref for ConstantDataWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Clone> std::ops::DerefMut for ConstantDataWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
