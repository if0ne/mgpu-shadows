use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, Range},
    sync::Arc,
};

use oxidx::dx::{self, IDevice, IGraphicsCommandList};

use crate::graphics::resources::Resource;

use super::{
    command_queue::{Compute, Graphics, Transfer, WorkerType},
    device::Device,
    resources::{NoGpuAccess, ResourceStates, StagingBuffer, StagingBufferDesc},
    worker_thread::WorkerThread,
};

#[derive(Clone, Debug)]
pub struct QueryHeap<T: QueryHeapType>(Arc<QueryHeapInner<T>>);

impl<T: QueryHeapType> QueryHeap<T> {
    pub(super) fn inner_new(device: &Device, count: usize) -> Self {
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

#[derive(Debug)]
pub struct QueryHeapInner<T: QueryHeapType> {
    raw: dx::QueryHeap,
    count: usize,
    staging_buffer: StagingBuffer<T::Type>,
    _market: PhantomData<T>,
}

pub trait QueryHeapType {
    const RAW: dx::QueryHeapType;
    const MUL: usize;

    type Type: Clone + Debug;

    fn desc(count: usize) -> dx::QueryHeapDesc;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimestampQuery<T: WorkerType>(PhantomData<T>);

impl QueryHeapType for TimestampQuery<Graphics> {
    const RAW: dx::QueryHeapType = dx::QueryHeapType::Timestamp;
    const MUL: usize = 2;
    type Type = u64;

    fn desc(count: usize) -> dx::QueryHeapDesc {
        dx::QueryHeapDesc::timestamp(count)
    }
}

impl QueryHeapType for TimestampQuery<Compute> {
    const RAW: dx::QueryHeapType = dx::QueryHeapType::Timestamp;
    const MUL: usize = 2;
    type Type = u64;

    fn desc(count: usize) -> dx::QueryHeapDesc {
        dx::QueryHeapDesc::timestamp(count)
    }
}

impl QueryHeapType for TimestampQuery<Transfer> {
    const RAW: dx::QueryHeapType = dx::QueryHeapType::CopyQueueTimestamp;
    const MUL: usize = 2;
    type Type = u64;

    fn desc(count: usize) -> dx::QueryHeapDesc {
        dx::QueryHeapDesc::copy_queue_timestamp(count)
    }
}

pub trait QueryResolver<T: QueryHeapType> {
    type Output;

    fn begin_query(&self, query: &QueryHeap<T>, index: usize);
    fn end_query(&self, query: &QueryHeap<T>, index: usize);

    fn resolve_query(&self, query: &QueryHeap<T>, range: Range<usize>) -> Vec<Self::Output>;
}

impl<T: WorkerType> QueryResolver<TimestampQuery<T>> for WorkerThread<T>
where
    TimestampQuery<T>: QueryHeapType<Type = u64>,
{
    type Output = f64;

    fn begin_query(&self, query: &QueryHeap<TimestampQuery<T>>, index: usize) {
        assert!(index < query.count);
        self.list.end_query(
            &query.raw,
            dx::QueryType::Timestamp,
            index * <TimestampQuery<T> as QueryHeapType>::MUL,
        );
    }

    fn end_query(&self, query: &QueryHeap<TimestampQuery<T>>, index: usize) {
        assert!(index < query.count);
        self.list.end_query(
            &query.raw,
            dx::QueryType::Timestamp,
            index * <TimestampQuery<T> as QueryHeapType>::MUL + 1,
        );
    }

    fn resolve_query(
        &self,
        query: &QueryHeap<TimestampQuery<T>>,
        range: Range<usize>,
    ) -> Vec<Self::Output> {
        assert!(range.end <= query.count);

        let start = range.start;
        let end = range.end;

        self.list.resolve_query_data(
            &query.raw,
            dx::QueryType::Timestamp,
            (start * 2)..(end * 2),
            query.staging_buffer.get_raw(),
            2 * start * size_of::<<TimestampQuery<T> as QueryHeapType>::Type>(),
        );

        let mut vec = vec![Default::default(); (end - start) * 2];
        query
            .staging_buffer
            .read_data(&mut vec, Some((2 * start)..(2 * end)));

        vec.chunks(2)
            .map(|chunk| (chunk[1] - chunk[0]) as f64 / self.frequency)
            .collect()
    }
}
