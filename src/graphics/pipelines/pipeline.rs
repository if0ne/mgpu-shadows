use std::{marker::PhantomData, sync::Arc};

use oxidx::dx::{self, IDevice};

use crate::graphics::{Device, GraphicsPipelineDesc};

use super::{Graphics, PipelineType};

#[derive(Clone, Debug)]
pub struct Pipeline<T: PipelineType>(Arc<PipelineInner<T>>);

#[derive(Debug)]
pub struct PipelineInner<T: PipelineType> {
    pub(crate) raw: dx::PipelineState,
    _marker: PhantomData<T>,
}

impl Pipeline<Graphics> {
    pub(crate) fn inner_new_graphics(device: &Device, desc: &GraphicsPipelineDesc) -> Self {
        let desc = &desc.as_raw();
        let raw = device.raw.create_graphics_pipeline(desc).unwrap();

        Self(Arc::new(PipelineInner {
            raw,
            _marker: PhantomData,
        }))
    }
}
