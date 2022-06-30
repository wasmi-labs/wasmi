//! Definitions for visualization of `wasmi` bytecode components.

use crate::{
    engine::{
        bytecode::ExecRegister,
        provider::RegisterOrImmediate,
        ConstRef,
        EngineInner,
        ExecProvider,
        Target,
    },
    Index as _,
};
use core::{fmt, fmt::Display};

/// Wrapper to display an [`ExecRegister`] in a human readable way.
#[derive(Debug)]
pub struct DisplayExecRegister {
    reg: ExecRegister,
}

impl From<ExecRegister> for DisplayExecRegister {
    fn from(reg: ExecRegister) -> Self {
        Self { reg }
    }
}

impl DisplayExecRegister {
    /// Creates a new [`DisplayExecRegister`] for the given register `index`.
    ///
    /// # Panics
    ///
    /// If the given register `index` is out of bounds.
    pub fn from_index(index: usize) -> Self {
        let index: u16 = index.try_into().unwrap_or_else(|error| {
            panic!("encountered invalid index {index} for register: {error}")
        });
        Self {
            reg: ExecRegister::from_inner(index),
        }
    }
}

impl Display for DisplayExecRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.reg.into_inner())
    }
}

/// Wrapper to display an [`ExecProvider`] in a human readable way.
#[derive(Debug)]
pub struct DisplayExecProvider<'engine> {
    engine: &'engine EngineInner,
    provider: ExecProvider,
}

impl<'engine> DisplayExecProvider<'engine> {
    pub fn new(engine: &'engine EngineInner, provider: ExecProvider) -> Self {
        Self { engine, provider }
    }
}

impl<'engine> Display for DisplayExecProvider<'engine> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.provider.decode() {
            RegisterOrImmediate::Register(reg) => {
                write!(f, "{}", DisplayExecRegister::from(reg))
            }
            RegisterOrImmediate::Immediate(imm) => {
                write!(f, "{}", DisplayConstRef::new(self.engine, imm))
            }
        }
    }
}

/// Wrapper to display an [`ConstRef`] in a human readable way.
#[derive(Debug)]
pub struct DisplayConstRef<'engine> {
    engine: &'engine EngineInner,
    cref: ConstRef,
}

impl<'engine> DisplayConstRef<'engine> {
    pub fn new(engine: &'engine EngineInner, cref: ConstRef) -> Self {
        Self { engine, cref }
    }
}

impl<'engine> Display for DisplayConstRef<'engine> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self
            .engine
            .res
            .const_pool
            .resolve(self.cref)
            .unwrap_or_default();
        // Note: We currently print all immediate values as bytes
        //       since `wasmi` bytecode does not store enough type
        //       information.
        write!(f, "0x{:X}", u64::from(value))
    }
}

/// Displays branching [`Target`] as human readable output.
pub struct DisplayTarget {
    target: Target,
}

impl From<Target> for DisplayTarget {
    fn from(target: Target) -> Self {
        Self { target }
    }
}

impl Display for DisplayTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.target.destination().into_usize())
    }
}
