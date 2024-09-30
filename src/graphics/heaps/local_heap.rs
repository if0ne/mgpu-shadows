use std::{marker::PhantomData, sync::Arc};

pub trait HeapType {}

pub struct GpuOnly;
impl HeapType for GpuOnly {}

pub struct CpuToGpu;
impl HeapType for CpuToGpu {}

pub struct GpuToCpu;
impl HeapType for GpuToCpu {}

#[derive(Debug)]
pub struct Allocation {
    offset: usize,
    size: usize,
}

#[derive(Clone, Debug)]
pub struct LocalHeap<T: HeapType>(Arc<LocalHeapInner<T>>);

#[derive(Debug)]
pub struct LocalHeapInner<T: HeapType> {
    size: usize,
    _marker: PhantomData<T>,
}
