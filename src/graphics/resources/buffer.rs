use std::fmt::Debug;

use atomig::Atomic;
use oxidx::dx;

use crate::graphics::heaps::Allocation;

use super::ResourceStates;

#[derive(Debug)]
pub struct BaseBuffer {
    pub(super) raw: dx::Resource,
    pub(super) size: usize,
    pub(super) state: Atomic<ResourceStates>,
    pub(super) flags: dx::ResourceFlags,
    pub(super) allocation: Option<Allocation>,
}
