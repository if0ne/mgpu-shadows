use std::{fmt::Debug, marker::PhantomData};

use oxidx::dx;

use crate::graphics::commands::{Compute, Graphics, Transfer, WorkerType};

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
