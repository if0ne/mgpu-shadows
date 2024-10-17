use std::sync::Arc;

use super::{GpuView, SamplerDesc, SamplerView, ViewAllocator};

#[derive(Clone, Debug)]
pub struct Sampler(Arc<SamplerInner>);

#[derive(Debug)]
pub struct SamplerInner {
    allocator: ViewAllocator,
    pub(crate) view: GpuView<SamplerView>,
}

impl Sampler {
    pub(crate) fn inner_new(view_allocator: ViewAllocator, desc: &SamplerDesc) -> Self {
        let view = view_allocator.push_sampler(&desc.as_raw());

        Self(Arc::new(SamplerInner {
            allocator: view_allocator,
            view,
        }))
    }
}

impl Drop for SamplerInner {
    fn drop(&mut self) {
        self.allocator.remove_sampler(self.view);
    }
}
