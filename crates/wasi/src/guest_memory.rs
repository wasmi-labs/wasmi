use wiggle::{borrow::BorrowChecker, BorrowHandle, GuestError, GuestMemory, Region};

/// Lightweight `wasmi::Memory` wrapper so we can implement the
/// `wiggle::GuestMemory` trait on it.
pub struct WasmiGuestMemory<'a> {
    mem: &'a mut [u8],
    bc: BorrowChecker,
}

impl<'a> WasmiGuestMemory<'a> {
    pub fn new(mem: &'a mut [u8]) -> Self {
        Self {
            mem,
            // Wiggle does not expose any methods for functions to re-enter
            // the WebAssembly instance, or expose the memory via non-wiggle
            // mechanisms. However, the user-defined code may end up
            // re-entering the instance, in which case this is an incorrect
            // implementation - we require exactly one BorrowChecker exist per
            // instance.
            bc: BorrowChecker::new(),
        }
    }
}

unsafe impl GuestMemory for WasmiGuestMemory<'_> {
    fn base(&self) -> (*mut u8, u32) {
        (self.mem.as_ptr() as *mut u8, self.mem.len() as u32)
    }
    fn has_outstanding_borrows(&self) -> bool {
        self.bc.has_outstanding_borrows()
    }
    fn is_shared_borrowed(&self, r: Region) -> bool {
        self.bc.is_shared_borrowed(r)
    }
    fn is_mut_borrowed(&self, r: Region) -> bool {
        self.bc.is_mut_borrowed(r)
    }
    fn shared_borrow(&self, r: Region) -> Result<BorrowHandle, GuestError> {
        self.bc.shared_borrow(r)
    }
    fn mut_borrow(&self, r: Region) -> Result<BorrowHandle, GuestError> {
        self.bc.mut_borrow(r)
    }
    fn shared_unborrow(&self, h: BorrowHandle) {
        self.bc.shared_unborrow(h)
    }
    fn mut_unborrow(&self, h: BorrowHandle) {
        self.bc.mut_unborrow(h)
    }
}
