use std::{fmt::Debug, marker::PhantomData, ops::Deref, sync::Arc};

use oxidx::dx::{self, IDevice};

use crate::graphics::{
    device::Device,
    resources::{NoGpuAccess, Resource, ResourceStates, StagingBuffer, StagingBufferDesc},
};

use super::QueryHeapType;

#[derive(Clone, Debug)]
pub struct QueryHeap<T: QueryHeapType>(Arc<QueryHeapInner<T>>);

#[derive(Debug)]
pub struct QueryHeapInner<T: QueryHeapType> {
    pub(crate) raw: dx::QueryHeap,
    pub(crate) count: usize,
    pub(crate) staging_buffer: StagingBuffer<T::Type>,
    _market: PhantomData<T>,
}

impl<T: QueryHeapType> QueryHeap<T> {
    pub(crate) fn inner_new(device: &Device, count: usize) -> Self {
        let raw = device
            .raw
            .create_query_heap(&T::desc(count * T::MUL))
            .unwrap();

        let staging_buffer = StagingBuffer::from_desc(
            device,
            StagingBufferDesc::new(count * T::MUL).readback(),
            NoGpuAccess,
            ResourceStates::CopyDst,
        );

        Self(Arc::new(QueryHeapInner {
            raw,
            count,
            staging_buffer,
            _market: PhantomData,
        }))
    }
}

impl<T: QueryHeapType> Deref for QueryHeap<T> {
    type Target = QueryHeapInner<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
