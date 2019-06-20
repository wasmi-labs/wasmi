extern crate atomic;

use alloc::sync::{Arc, RwLock};

pub use self::atomic::{Atomic, Ordering::Relaxed as Ordering};
pub use alloc::sync::{
    Arc as MyRc, RwLockReadGuard as MyRefRead, RwLockWriteGuard as MyRefWrite, Weak as MyWeak,
};

/// Thread-safe wrapper which can be used in place of a `RefCell`.
#[derive(Debug)]
pub struct MyRefCell<T>(Arc<RwLock<T>>);

impl<T> MyRefCell<T> {
    /// Create new wrapper object.
    pub fn new(obj: T) -> MyRefCell<T> {
        MyRefCell(Arc::new(RwLock::new(obj)))
    }

    /// Borrow a `MyRef` to the inner value.
    pub fn borrow(&self) -> ::MyRefRead<T> {
        self.0
            .read()
            .expect("failed to acquire lock while trying to borrow")
    }

    /// Borrow a mutable `MyRef` to the inner value.
    pub fn borrow_mut(&self) -> ::MyRefWrite<T> {
        self.0
            .write()
            .expect("failed to acquire lock while trying to borrow mutably")
    }
}

/// Thread-safe wrapper which can be used in place of a `Cell`.
#[derive(Debug)]
pub struct MyCell<T>(Atomic<T>)
where
    T: Copy;

impl<T> MyCell<T>
where
    T: Copy,
{
    /// Create new wrapper object.
    pub fn new(obj: T) -> MyCell<T> {
        MyCell(Atomic::new(obj))
    }

    /// Returns the inner value.
    pub fn get(&self) -> T {
        self.0.load(::Ordering)
    }

    /// Sets the inner value.
    pub fn set(&self, val: T) {
        self.0.store(val, ::Ordering);
    }
}
