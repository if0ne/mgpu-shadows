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
