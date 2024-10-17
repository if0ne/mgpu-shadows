use crate::graphics::{GraphicsPipelineDesc, Sealed};

pub trait PipelineType: Sealed {
    type Desc;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Graphics;

impl Sealed for Graphics {}
impl PipelineType for Graphics {
    type Desc = GraphicsPipelineDesc;
}
