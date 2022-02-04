use super::InitExpr;
use crate::{GlobalType, ModuleError};

/// The index of a global variable within a [`Module`].
///
/// [`Module`]: [`super::Module`]
#[derive(Debug, Copy, Clone)]
pub struct GlobalIdx(pub(super) u32);

impl GlobalIdx {
    /// Returns the [`GlobalIdx`] as `u32`.
    pub fn into_u32(self) -> u32 {
        self.0
    }

    /// Returns the [`GlobalIdx`] as `usize`.
    pub fn into_usize(self) -> usize {
        self.0 as usize
    }
}

/// A global variable definition within a [`Module`].
///
/// [`Module`]: [`super::Module`]
#[derive(Debug)]
pub struct Global {
    /// The type of the global variable.
    global_type: GlobalType,
    /// The initial value of the global variabel.
    ///
    /// # Note
    ///
    /// This is represented by a so called initializer expression
    /// that is run at module instantiation time.
    init_expr: InitExpr,
}

impl TryFrom<wasmparser::Global<'_>> for Global {
    type Error = ModuleError;

    fn try_from(global: wasmparser::Global<'_>) -> Result<Self, Self::Error> {
        let global_type = global.ty.try_into()?;
        let init_expr = global.init_expr.try_into()?;
        Ok(Global {
            global_type,
            init_expr,
        })
    }
}

impl Global {
    /// Returns the [`GlobalType`] of the global variable.
    pub fn global_type(&self) -> &GlobalType {
        &self.global_type
    }

    /// Returns the [`InitExpr`] of the global variable.
    pub fn init_expr(&self) -> &InitExpr {
        &self.init_expr
    }

    /// Splits the [`Global`] into its global type and its global initializer.
    pub fn into_type_and_init(self) -> (GlobalType, InitExpr) {
        (self.global_type, self.init_expr)
    }
}
