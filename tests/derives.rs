extern crate wasmi_derive;
extern crate wasmi;

use wasmi_derive::derive_externals;
use wasmi::HostError;
use std::fmt;

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

    pub fn fart(&self, inbound_fart: Fart) -> Result<Fart, NoInfoError> {
        Ok(inbound_fart)
    }
}

pub struct Fart;
