use std::marker::PhantomData;

use oxidx::dx;

use crate::graphics::Sealed;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GpuView<T: ViewType> {
    pub(crate) index: usize,
    pub(crate) gpu: dx::GpuDescriptorHandle,
    pub(crate) cpu: dx::CpuDescriptorHandle,
    pub(crate) _marker: PhantomData<T>,
}

impl<T: ViewType> GpuView<T> {
    pub fn gpu(&self) -> dx::GpuDescriptorHandle {
        self.gpu
    }

    pub fn cpu(&self) -> dx::CpuDescriptorHandle {
        self.cpu
    }
}

pub trait ViewType: Sealed {
    const RAW_TYPE: dx::DescriptorHeapType;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RtvView;
impl Sealed for RtvView {}
impl ViewType for RtvView {
    const RAW_TYPE: dx::DescriptorHeapType = dx::DescriptorHeapType::Rtv;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc {
        dx::DescriptorHeapDesc::rtv(num)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DsvView;
impl Sealed for DsvView {}
impl ViewType for DsvView {
    const RAW_TYPE: dx::DescriptorHeapType = dx::DescriptorHeapType::Dsv;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc {
        dx::DescriptorHeapDesc::dsv(num)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CbvSrvUavView;
impl Sealed for CbvSrvUavView {}
impl ViewType for CbvSrvUavView {
    const RAW_TYPE: dx::DescriptorHeapType = dx::DescriptorHeapType::CbvSrvUav;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc {
        dx::DescriptorHeapDesc::cbr_srv_uav(num)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CbvView;
impl Sealed for CbvView {}
impl ViewType for CbvView {
    const RAW_TYPE: dx::DescriptorHeapType = dx::DescriptorHeapType::CbvSrvUav;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc {
        dx::DescriptorHeapDesc::cbr_srv_uav(num)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SrvView;
impl Sealed for SrvView {}
impl ViewType for SrvView {
    const RAW_TYPE: dx::DescriptorHeapType = dx::DescriptorHeapType::CbvSrvUav;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc {
        dx::DescriptorHeapDesc::cbr_srv_uav(num)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UavView;
impl Sealed for UavView {}
impl ViewType for UavView {
    const RAW_TYPE: dx::DescriptorHeapType = dx::DescriptorHeapType::CbvSrvUav;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc {
        dx::DescriptorHeapDesc::cbr_srv_uav(num)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SamplerView;
impl Sealed for SamplerView {}
impl ViewType for SamplerView {
    const RAW_TYPE: dx::DescriptorHeapType = dx::DescriptorHeapType::Sampler;

    fn get_desc(num: usize) -> dx::DescriptorHeapDesc {
        dx::DescriptorHeapDesc::sampler(num)
    }
}
