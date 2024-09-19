use std::ptr::NonNull;

use oxidx::dx::{self, IDevice, IResource};

#[derive(Debug)]
pub struct ConstantBuffer<T: Clone + Copy> {
    buffer: dx::Resource,
    mapped_data: NonNull<ConstantDataWrapper<T>>,
    size: usize,
}

impl<T: Clone + Copy> ConstantBuffer<T> {
    pub fn new(device: &dx::Device, size: usize) -> Self {
        let element_byte_size = size_of::<ConstantDataWrapper<T>>();
        Self::new_inner(device, size, element_byte_size)
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

impl<T: Clone + Copy> ConstantBuffer<T> {
    fn new_inner(device: &dx::Device, size: usize, element_byte_size: usize) -> Self {
        let resource: dx::Resource = device
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
}

impl<T: Clone + Copy> Drop for ConstantBuffer<T> {
    fn drop(&mut self) {
        self.buffer.unmap(0, None);
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(align(256))]
struct ConstantDataWrapper<T>(pub T);

impl<T> std::ops::Deref for ConstantBuffer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for ConstantBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
