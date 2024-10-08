use std::{ops::{Deref, DerefMut}, ptr::NonNull};

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
