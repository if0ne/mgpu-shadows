use std::fmt::Debug;

use oxidx::dx;

use crate::graphics::heaps::{Allocation, HeapType};

pub trait Buffer {}

#[derive(Debug)]
pub struct BaseBuffer<H: HeapType> {
    pub(super) raw: dx::Resource,
    pub(super) state: dx::ResourceStates,
    pub(super) allocation: Option<Allocation<H>>,
}

impl<H: HeapType> BaseBuffer<H> {
    pub fn resource(&self) -> &dx::Resource {
        &self.raw
    }
}
