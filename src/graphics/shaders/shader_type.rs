use std::ffi::CStr;

use crate::graphics::Sealed;

pub trait ShaderType: Sealed {
    const TARGET: &'static CStr;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Vertex;

impl Sealed for Vertex {}
impl ShaderType for Vertex {
    const TARGET: &'static CStr = c"vs_5_1";
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pixel;

impl Sealed for Pixel {}
impl ShaderType for Pixel {
    const TARGET: &'static CStr = c"ps_5_1";
}
