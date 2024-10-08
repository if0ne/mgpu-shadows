use std::fmt::Debug;

use oxidx::dx;
use parking_lot::Mutex;

use crate::graphics::heaps::Allocation;

pub trait Buffer {}

#[derive(Debug)]
pub struct BaseBuffer {
    pub(super) raw: dx::Resource,
    pub(super) size: usize,
    pub(super) state: Mutex<dx::ResourceStates>,
    pub(super) flags: dx::ResourceFlags,
    pub(super) allocation: Option<Allocation>,
}
