use std::fmt::Debug;

use oxidx::dx;

use crate::graphics::heaps::{Allocation, HeapType};

pub trait BufferType: Debug {}

#[derive(Debug)]
pub struct Buffer<T: BufferType, H: HeapType> {
    pub(super) raw: dx::Resource,
    pub(super) state: dx::ResourceStates,
    pub(super) allocation: Option<Allocation<H>>,
    pub(super) inner: T,
}

impl<T: BufferType, H: HeapType> Buffer<T, H> {
    pub fn resource(&self) -> &dx::Resource {
        &self.raw
    }
}
