use std::{fmt::Debug, marker::PhantomData};

use oxidx::dx;

use crate::graphics::{
    commands::{Compute, Direct, Transfer, WorkerType},
    Sealed,
};

pub trait QueryHeapType: Sealed {
    const RAW: dx::QueryHeapType;
    const MUL: usize;

    type Type: Clone + Debug;

    fn desc(count: usize) -> dx::QueryHeapDesc;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimestampQuery<T: WorkerType>(PhantomData<T>);

impl<T: WorkerType> Sealed for TimestampQuery<T> {}

impl QueryHeapType for TimestampQuery<Direct> {
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
