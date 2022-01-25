use super::InitExpr;
use crate::{GlobalType, ModuleError};

/// The index of a global variable within a [`Module`].
///
/// [`Module`]: [`super::Module`]
#[derive(Debug)]
pub struct GlobalIdx(pub(super) u32);

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
}
