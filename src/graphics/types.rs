use atomig::Atom;
use oxidx::dx;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryHeapType {
    Gpu,
    Cpu,
    Readback,
    Shared,
}

impl MemoryHeapType {
    pub(crate) fn flags(&self) -> dx::HeapFlags {
        match self {
            MemoryHeapType::Shared => dx::HeapFlags::Shared | dx::HeapFlags::SharedCrossAdapter,
            _ => dx::HeapFlags::empty(),
        }
    }

    pub(crate) fn as_raw(self) -> dx::HeapType {
        match self {
            MemoryHeapType::Gpu => dx::HeapType::Default,
            MemoryHeapType::Cpu => dx::HeapType::Upload,
            MemoryHeapType::Readback => dx::HeapType::Readback,
            MemoryHeapType::Shared => dx::HeapType::Default,
        }
    }
}

#[derive(Debug)]
pub struct BufferCopyableFootprints {
    size: usize,
}

impl BufferCopyableFootprints {
    pub fn new(size: usize) -> Self {
        Self { size }
    }

    pub fn total_size(&self) -> usize {
        self.size
    }
}

#[derive(Debug)]
pub struct TextureCopyableFootprints {
    size: usize,
    mip_levels: usize,
    subresources: Vec<MipInfo>,
}

impl TextureCopyableFootprints {
    pub fn new(size: usize, mip_levels: usize, subresources: Vec<MipInfo>) -> Self {
        Self {
            size,
            mip_levels,
            subresources,
        }
    }

    pub fn total_size(&self) -> usize {
        self.size
    }

    pub fn subresource_info(&self, subresource: SubresourceIndex) -> &MipInfo {
        &self.subresources[subresource.mip_index + subresource.array_index * self.mip_levels]
    }
}

#[derive(Debug)]
pub struct MipInfo {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub row_size: usize,
    pub size: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextureUsage {
    RenderTarget {
        color: Option<[f32; 4]>,
        srv: bool,
        uav: bool,
    },
    DepthTarget {
        color: Option<(f32, u8)>,
        srv: bool,
    },
    ShaderResource,
    Storage,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SubresourceIndex {
    pub array_index: usize,
    pub mip_index: usize,
}

#[derive(Clone, Debug)]
pub struct SwapchainDesc {
    pub width: u32,
    pub height: u32,
    pub format: dx::Format,
    pub buffer_count: usize,
    pub present_mode: PresentMode,
}

#[derive(Clone, Debug)]
pub enum PresentMode {
    Immediate,
    Mailbox,
    Fifo,
}

impl SwapchainDesc {
    pub(crate) fn as_raw(&self) -> dx::SwapchainDesc1 {
        dx::SwapchainDesc1::new(self.width, self.height)
            .with_format(self.format)
            .with_buffer_count(self.buffer_count)
            .with_usage(dx::FrameBufferUsage::RenderTargetOutput)
            .with_scaling(dx::Scaling::Stretch)
            .with_swap_effect(dx::SwapEffect::FlipDiscard)
            .with_flags(
                dx::SwapchainFlags::AllowTearing | dx::SwapchainFlags::FrameLatencyWaitableObject,
            )
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct ResourceStates: i32 {
        const Common = dx::ResourceStates::Common.bits();
        const VertexAndConstantBuffer = dx::ResourceStates::VertexAndConstantBuffer.bits();
        const IndexBuffer =  dx::ResourceStates::IndexBuffer.bits();
        const RenderTarget =  dx::ResourceStates::RenderTarget.bits();
        const UnorderedAccess = dx::ResourceStates::UnorderedAccess.bits();
        const DepthWrite =  dx::ResourceStates::DepthWrite.bits();
        const DepthRead = dx::ResourceStates::DepthRead.bits();
        const NonPixelShaderResource = dx::ResourceStates::NonPixelShaderResource.bits();
        const PixelShaderResource = dx::ResourceStates::PixelShaderResource.bits();
        const CopyDst = dx::ResourceStates::CopyDest.bits();
        const CopySrc = dx::ResourceStates::CopySource.bits();
        const GenericRead = dx::ResourceStates::GenericRead.bits();
        const AllShaderResource = dx::ResourceStates::AllShaderResource.bits();
        const Present = dx::ResourceStates::Present.bits();
    }
}

impl ResourceStates {
    pub(crate) fn as_raw(&self) -> dx::ResourceStates {
        dx::ResourceStates::from_bits(self.bits()).unwrap()
    }
}

impl Atom for ResourceStates {
    type Repr = i32;

    fn pack(self) -> Self::Repr {
        self.bits()
    }

    fn unpack(src: Self::Repr) -> Self {
        ResourceStates::from_bits(src).unwrap()
    }
}
