use std::ops::Range;

use oxidx::dx::{self, IGraphicsCommandList};

use crate::graphics::{
    commands::{WorkerThread, WorkerType},
    Resource,
};

use super::{QueryHeap, QueryHeapType, TimestampQuery};

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
