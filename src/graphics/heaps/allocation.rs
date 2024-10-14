use super::MemoryHeap;

#[derive(Debug)]
pub struct Allocation {
    pub(crate) heap: MemoryHeap,
    pub(crate) offset: usize,
    pub(crate) size: usize,
}
