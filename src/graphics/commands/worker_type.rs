use oxidx::dx;

use crate::graphics::Sealed;

pub trait WorkerType: Sealed {
    const RAW_TYPE: dx::CommandListType;

    fn queue_desc() -> dx::CommandQueueDesc;
}

#[derive(Clone, Copy, Debug)]
pub struct Direct;
impl Sealed for Direct {}
impl WorkerType for Direct {
    const RAW_TYPE: dx::CommandListType = dx::CommandListType::Direct;

    fn queue_desc() -> dx::CommandQueueDesc {
        dx::CommandQueueDesc::direct()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Compute;
impl Sealed for Compute {}
impl WorkerType for Compute {
    const RAW_TYPE: dx::CommandListType = dx::CommandListType::Compute;

    fn queue_desc() -> dx::CommandQueueDesc {
        dx::CommandQueueDesc::compute()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Transfer;
impl Sealed for Transfer {}
impl WorkerType for Transfer {
    const RAW_TYPE: dx::CommandListType = dx::CommandListType::Copy;

    fn queue_desc() -> dx::CommandQueueDesc {
        dx::CommandQueueDesc::copy()
    }
}
