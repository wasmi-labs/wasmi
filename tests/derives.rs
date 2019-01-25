extern crate wasmi;
extern crate wasmi_derive;

use std::fmt;
use wasmi::HostError;
use wasmi_derive::derive_externals;

#[derive(Debug)]
struct NoInfoError;
impl HostError for NoInfoError {}
impl fmt::Display for NoInfoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NoInfoError")
    }
}

struct NonStaticExternals<'a> {
    state: &'a mut usize,
}

#[derive_externals]
impl<'a> NonStaticExternals<'a> {
    pub fn hello(&self, a: u32, b: u32) -> u32 {
        a + b
    }

    pub fn increment(&mut self) {
        *self.state += 1;
    }

    pub fn traps(&self) -> Result<(), NoInfoError> {
        Err(NoInfoError)
    }
}
