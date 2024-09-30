use oxidx::dx;

use crate::graphics::heaps::HeapType;

pub trait BufferType {}

pub struct Buffer<T> {
    pub(super) resource: dx::Resource,
    pub(super) state: dx::ResourceStates,
    pub(super) ext: T,
}
