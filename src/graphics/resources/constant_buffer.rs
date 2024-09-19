use std::ptr::NonNull;

use oxidx::dx::{self, IDevice, IResource};

use crate::graphics::device::Device;

#[derive(Debug)]
pub struct ConstantBuffer<T: Clone + Copy> {
    buffer: dx::Resource,
    mapped_data: NonNull<ConstantDataWrapper<T>>,
    size: usize,
}

impl<T: Clone + Copy> ConstantBuffer<T> {
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
            buffer: resource,
            mapped_data,
            size,
        }
    }

    pub fn resource(&self) -> &dx::Resource {
        &self.buffer
    }

    pub fn read(&self, index: usize) -> T {
        if index >= self.size {
            panic!(
                "ConstantBuffer<{}>: index out of bounds, length: {}",
                std::any::type_name::<T>(),
                self.size
            );
        }

        unsafe { std::ptr::read(self.mapped_data.add(index).as_mut()).0 }
    }

    pub fn write(&self, index: usize, src: impl ToOwned<Owned = T>) {
        if index >= self.size {
            panic!(
                "ConstantBuffer<{}>: index out of bounds, length: {}",
                std::any::type_name::<T>(),
                self.size
            );
        }

        unsafe {
            std::ptr::write(
                self.mapped_data.add(index).as_mut(),
                ConstantDataWrapper(src.to_owned()),
            )
        }
    }

    pub fn as_slice(&self) -> &[ConstantDataWrapper<T>] {
        unsafe {
            std::slice::from_raw_parts(self.mapped_data.as_ptr() as *const _, self.size)
        }
    }

    pub fn as_slice_mut(&mut self) -> &mut [ConstantDataWrapper<T>] {
        unsafe {
            std::slice::from_raw_parts_mut(self.mapped_data.as_ptr(), self.size)
        }
    }
}

impl<T: Clone + Copy> Drop for ConstantBuffer<T> {
    fn drop(&mut self) {
        self.buffer.unmap(0, None);
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(align(256))]
struct ConstantDataWrapper<T>(pub T);

impl<T: Clone + Copy> std::ops::Deref for ConstantDataWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Clone + Copy> std::ops::DerefMut for ConstantDataWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
