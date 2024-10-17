use std::{ffi::CString, marker::PhantomData, ops::Deref, path::Path, sync::Arc};

use oxidx::dx::{self, IBlobExt};
use smallvec::SmallVec;

use super::ShaderType;

#[derive(Clone, Debug)]
pub struct Shader<T: ShaderType>(Arc<ShaderInner<T>>);

#[derive(Debug)]
pub struct ShaderInner<T: ShaderType> {
    pub(crate) raw: dx::Blob,
    _marker: PhantomData<T>,
}

impl<T: ShaderType> Deref for Shader<T> {
    type Target = ShaderInner<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ShaderType> Shader<T> {
    pub(crate) fn inner_new(
        path: impl AsRef<Path>,
        entry_point: impl AsRef<str>,
        defines: &[(&'static str, &'static str)],
    ) -> Self {
        let entry_point = CString::new(entry_point.as_ref()).unwrap();

        let defines = defines
            .iter()
            .map(|(name, key)| (CString::new(*name).unwrap(), CString::new(*key).unwrap()))
            .collect::<SmallVec<[_; 4]>>();

        let defines = if !defines.is_empty() {
            defines
                .iter()
                .map(|(name, key)| dx::ShaderMacro::new(name, key))
                .chain(std::iter::once(dx::ShaderMacro::default()))
                .collect::<SmallVec<[_; 4]>>()
        } else {
            Default::default()
        };

        let raw = dx::Blob::compile_from_file(
            path,
            &defines,
            &entry_point,
            T::TARGET,
            dx::COMPILE_DEBUG | dx::COMPILE_SKIP_OPT,
            0,
        )
        .unwrap();

        Self(Arc::new(ShaderInner {
            raw,
            _marker: PhantomData,
        }))
    }
}
