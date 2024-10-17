use oxidx::dx::{self, IDevice};
use smallvec::SmallVec;

use super::{BindingType, Device, StaticSampler};

#[derive(Clone, Debug)]
pub struct PipelineLayout {
    pub(crate) raw: dx::RootSignature,
}

impl PipelineLayout {
    pub(crate) fn inner_new(
        device: &Device,
        layout: &[BindingType],
        static_samplers: &[StaticSampler],
    ) -> Self {
        let ranges = layout
            .iter()
            .map(|i| i.get_ranges())
            .collect::<SmallVec<[_; 4]>>();

        let params = layout
            .iter()
            .zip(ranges.iter())
            .map(|(i, ranges)| i.as_raw(ranges))
            .collect::<SmallVec<[_; 4]>>();
        let samplers = static_samplers
            .iter()
            .map(|i| i.as_raw())
            .collect::<SmallVec<[_; 4]>>();

        let desc = dx::RootSignatureDesc::default()
            .with_parameters(&params)
            .with_sampler(&samplers)
            .with_flags(dx::RootSignatureFlags::AllowInputAssemblerInputLayout);

        let raw = device
            .raw
            .serialize_and_create_root_signature(&desc, dx::RootSignatureVersion::V1_0, 0)
            .unwrap();

        Self { raw }
    }
}
