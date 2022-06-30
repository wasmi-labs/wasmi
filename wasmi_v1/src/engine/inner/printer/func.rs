//! Definitions for visualization of `wasmi` function.

use super::{
    DisplayExecInstruction,
    DisplayExecRegister,
    DisplaySequence,
    DisplaySlice,
    EngineInner,
};
use crate::{
    engine::bytecode::ExecRegister,
    func,
    func::FuncEntityInternal,
    module,
    AsContext,
    Func,
    FuncType,
    Index as _,
    StoreContext,
};
use core::{fmt, fmt::Display};
use wasmi_core::ValueType;

/// Wrapper to display an entire `wasmi` bytecode function in human readable fashion.
pub struct DisplayFunc<'ctx, 'engine, T> {
    ctx: StoreContext<'ctx, T>,
    engine: &'engine EngineInner,
    func: Func,
}

impl<'ctx, 'engine, T> DisplayFunc<'ctx, 'engine, T> {
    /// Creates a new display wrapper for a `wasmi` function.
    pub fn new(ctx: StoreContext<'ctx, T>, engine: &'engine EngineInner, func: Func) -> Self {
        Self { ctx, engine, func }
    }
}

impl<'ctx, 'engine, T> Display for DisplayFunc<'ctx, 'engine, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (func_body, instance) = match self.func.as_internal(&self.ctx) {
            FuncEntityInternal::Wasm(wasm_func) => (wasm_func.func_body(), wasm_func.instance()),
            FuncEntityInternal::Host(_host_func) => {
                todo!()
            }
        };
        let dedup_func_type = match self.func.as_internal(&self.ctx) {
            FuncEntityInternal::Wasm(wasm_func) => wasm_func.signature(),
            FuncEntityInternal::Host(host_func) => host_func.signature(),
        };
        let func_type = self.engine.resolve_func_type(dedup_func_type, Clone::clone);
        let len_params = func_type.params().len();
        let len_regs = func_body.len_regs();
        let len_locals = len_regs as usize - len_params;
        let func_body = self.engine.code_map.resolve(func_body);
        let func_idx = self.ctx.store.resolve_func_idx(self.func);
        writeln!(
            f,
            "{}: {} -> {}",
            DisplayFuncIdx::from(func_idx),
            DisplayParams::new(func_type.params()),
            DisplaySlice::from(func_type.results()),
        )?;
        write!(f, "{}", DisplayLocals::new(len_params, len_locals))?;
        for (n, instr) in func_body.iter().enumerate() {
            write!(
                f,
                "{:5}    {}",
                n,
                DisplayExecInstruction::new(
                    self.ctx.as_context(),
                    &self.engine.res,
                    instance,
                    instr
                )
            )?;
        }
        Ok(())
    }
}

/// Displays a [`FuncType`] in a human readable fashion.
pub struct DisplayFuncType<'a> {
    func_type: &'a FuncType,
}

impl<'a> From<&'a FuncType> for DisplayFuncType<'a> {
    fn from(func_type: &'a FuncType) -> Self {
        Self { func_type }
    }
}

impl Display for DisplayFuncType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} -> {}",
            DisplaySlice::from(self.func_type.params()),
            DisplaySlice::from(self.func_type.results()),
        )
    }
}

/// Wrapper to display `wasmi` bytecode function parameters in human readable fashion.
struct DisplayParams<'a> {
    params: &'a [ValueType],
}

impl<'a> DisplayParams<'a> {
    pub fn new(params: &'a [ValueType]) -> Self {
        Self { params }
    }
}

impl<'a> Display for DisplayParams<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            DisplaySequence::from(
                self.params
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|(n, value_type)| DisplayParam::new(n, value_type))
            )
        )
    }
}

/// Wrapper to display a single `wasmi` bytecode function parameter in human readable fashion.
struct DisplayParam {
    reg: ExecRegister,
    value_type: ValueType,
}

impl DisplayParam {
    /// Creates a new display parameter from the given inputs.
    ///
    /// # Panics
    ///
    /// If the register `index` is out of bounds.
    pub fn new(index: usize, value_type: ValueType) -> Self {
        let ridx = index
            .try_into()
            .unwrap_or_else(|error| panic!("register index {index} out of bounds: {error}"));
        Self {
            reg: ExecRegister::from_inner(ridx),
            value_type,
        }
    }
}

impl Display for DisplayParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}",
            DisplayExecRegister::from(self.reg),
            self.value_type
        )
    }
}

/// Wrapper to display `wasmi` bytecode function locals in human readable fashion.
struct DisplayLocals {
    len_params: usize,
    len_locals: usize,
}

impl DisplayLocals {
    pub fn new(len_params: usize, len_locals: usize) -> Self {
        assert!(len_params + len_locals < (u16::MAX as usize));
        Self {
            len_params,
            len_locals,
        }
    }
}

impl Display for DisplayLocals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut locals = (0..self.len_locals).map(|local| local + self.len_params);
        if let Some(fst) = locals.next() {
            write!(f, "         local {}", DisplayExecRegister::from_index(fst))?;
        }
        while let Some(next) = locals.next() {
            write!(f, ", {}", DisplayExecRegister::from_index(next))?;
        }
        if self.len_locals > 0 {
            writeln!(f)?;
        }
        Ok(())
    }
}

/// Display wrapper for `wasmi` [`FuncIdx`] function references.
pub struct DisplayFuncIdx {
    func_idx: usize,
}

impl From<func::FuncIdx> for DisplayFuncIdx {
    fn from(func_idx: func::FuncIdx) -> Self {
        Self {
            func_idx: func_idx.into_usize(),
        }
    }
}

impl From<module::FuncIdx> for DisplayFuncIdx {
    fn from(func_idx: module::FuncIdx) -> Self {
        Self {
            func_idx: func_idx.into_usize(),
        }
    }
}

impl Display for DisplayFuncIdx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "func({})", self.func_idx.into_usize())
    }
}
