use core::ops::Deref;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct RefAccess<T>(T);

impl<T> RefAccess<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }

    pub unsafe fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for RefAccess<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
