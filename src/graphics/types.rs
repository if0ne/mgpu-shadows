use atomig::Atom;
use oxidx::dx;
use smallvec::SmallVec;

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

#[derive(Clone, Debug)]
pub enum BindingType<'a> {
    Cbv {
        slot: u32,
        space: u32,
        visibility: dx::ShaderVisibility,
    },
    Srv {
        slot: u32,
        space: u32,
        visibility: dx::ShaderVisibility,
    },
    Uav {
        slot: u32,
        space: u32,
        visibility: dx::ShaderVisibility,
    },
    PushConstant {
        slot: u32,
        space: u32,
        count: u32,
        visibility: dx::ShaderVisibility,
    },
    Table {
        entries: &'a [BindingTable],
        visibility: dx::ShaderVisibility,
    },
}

impl<'a> BindingType<'a> {
    pub(crate) fn as_raw<'b>(&self, ranges: &'b [dx::DescriptorRange]) -> dx::RootParameter<'b> {
        match self {
            BindingType::Cbv {
                slot,
                space,
                visibility,
            } => dx::RootParameter::cbv(*slot, *space).with_visibility(*visibility),
            BindingType::Srv {
                slot,
                space,
                visibility,
            } => dx::RootParameter::srv(*slot, *space).with_visibility(*visibility),
            BindingType::Uav {
                slot,
                space,
                visibility,
            } => dx::RootParameter::uav(*slot, *space).with_visibility(*visibility),
            BindingType::PushConstant {
                slot,
                space,
                count,
                visibility,
            } => dx::RootParameter::constant_32bit(*slot, *space, *count)
                .with_visibility(*visibility),
            BindingType::Table { visibility, .. } => {
                dx::RootParameter::descriptor_table(ranges).with_visibility(*visibility)
            }
        }
    }

    pub(crate) fn get_ranges(&self) -> SmallVec<[dx::DescriptorRange; 4]> {
        match self {
            BindingType::Table { entries, .. } => entries.into_iter().map(|e| e.as_raw()).collect(),
            _ => Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum BindingTable {
    Cbv { slot: u32, space: u32, count: u32 },
    Srv { slot: u32, space: u32, count: u32 },
    Uav { slot: u32, space: u32, count: u32 },
    Sampler { slot: u32, space: u32, count: u32 },
}

impl BindingTable {
    pub(crate) fn as_raw(&self) -> dx::DescriptorRange {
        match self {
            BindingTable::Cbv { slot, space, count } => dx::DescriptorRange::cbv(*count)
                .with_base_shader_register(*slot)
                .with_register_space(*space),
            BindingTable::Srv { slot, space, count } => dx::DescriptorRange::srv(*count)
                .with_base_shader_register(*slot)
                .with_register_space(*space),
            BindingTable::Uav { slot, space, count } => dx::DescriptorRange::uav(*count)
                .with_base_shader_register(*slot)
                .with_register_space(*space),
            BindingTable::Sampler { slot, space, count } => dx::DescriptorRange::sampler(*count)
                .with_base_shader_register(*slot)
                .with_register_space(*space),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StaticSampler {}

impl StaticSampler {
    pub(crate) fn as_raw(&self) -> dx::StaticSamplerDesc {
        dx::StaticSamplerDesc::default()
    }
}

#[derive(Clone, Debug)]
pub struct SamplerDesc {}

impl SamplerDesc {
    pub(crate) fn as_raw(&self) -> dx::SamplerDesc {
        dx::SamplerDesc::default()
    }
}
