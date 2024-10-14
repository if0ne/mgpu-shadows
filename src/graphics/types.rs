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
