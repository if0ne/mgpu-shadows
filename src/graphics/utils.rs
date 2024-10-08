use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use super::resources::SubresourceIndex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NonNullSend<T>(NonNull<T>);

impl<T> Deref for NonNullSend<T> {
    type Target = NonNull<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for NonNullSend<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

unsafe impl<T> Send for NonNullSend<T> {}

impl<T> From<NonNull<T>> for NonNullSend<T> {
    fn from(value: NonNull<T>) -> Self {
        Self(value)
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
