//! Definitions for visualization of `wasmi` bytecode components.

use super::DisplaySequence;
use crate::{
    engine::{
        bytecode::{ExecRegister, Global},
        inner::EngineResources,
        provider::RegisterOrImmediate,
        ConstRef,
        ExecProvider,
        ExecProviderSlice,
        ExecRegisterSlice,
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
    res: &'engine EngineResources,
    provider: ExecProvider,
}

impl<'engine> DisplayExecProvider<'engine> {
    pub fn new(res: &'engine EngineResources, provider: ExecProvider) -> Self {
        Self { res, provider }
    }
}

impl<'engine> Display for DisplayExecProvider<'engine> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.provider.decode() {
            RegisterOrImmediate::Register(reg) => {
                write!(f, "{}", DisplayExecRegister::from(reg))
            }
            RegisterOrImmediate::Immediate(imm) => {
                write!(f, "{}", DisplayConstRef::new(self.res, imm))
            }
        }
    }
}

/// Wrapper to display an [`ConstRef`] in a human readable way.
#[derive(Debug)]
pub struct DisplayConstRef<'engine> {
    res: &'engine EngineResources,
    cref: ConstRef,
}

impl<'engine> DisplayConstRef<'engine> {
    pub fn new(res: &'engine EngineResources, cref: ConstRef) -> Self {
        Self { res, cref }
    }
}

impl<'engine> Display for DisplayConstRef<'engine> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.res.const_pool.resolve(self.cref).unwrap_or_default();
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

/// Display wrapper for `wasmi` bytecode [`Global`] variables.
pub struct DisplayGlobal {
    global: Global,
}

impl From<Global> for DisplayGlobal {
    fn from(global: Global) -> Self {
        Self { global }
    }
}

impl Display for DisplayGlobal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "global({})", self.global.into_inner())
    }
}

/// Display wrapper for `wasmi` bytecode [`ExecRegisterSlice`].
pub struct DisplayExecRegisterSlice {
    slice: ExecRegisterSlice,
}

impl From<ExecRegisterSlice> for DisplayExecRegisterSlice {
    fn from(slice: ExecRegisterSlice) -> Self {
        Self { slice }
    }
}

impl Display for DisplayExecRegisterSlice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            DisplaySequence::from(self.slice.iter().map(DisplayExecRegister::from))
        )
    }
}

/// Display wrapper for `wasmi` bytecode [`ExecProviderSlice`].
pub struct DisplayExecProviderSlice<'engine> {
    res: &'engine EngineResources,
    slice: ExecProviderSlice,
}

impl<'engine> DisplayExecProviderSlice<'engine> {
    /// Creates a new display wrapper for [`ExecProviderSlice`].
    pub fn new(res: &'engine EngineResources, slice: ExecProviderSlice) -> Self {
        Self { res, slice }
    }
}

impl Display for DisplayExecProviderSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            DisplaySequence::from(
                self.res
                    .provider_slices
                    .resolve(self.slice)
                    .iter()
                    .copied()
                    .map(|result| DisplayExecProvider::new(self.res, result)),
            )
        )
    }
}
