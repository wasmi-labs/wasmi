use super::InitExpr;
use crate::GlobalType;

/// The index of a global variable within a [`Module`].
///
/// [`Module`]: [`super::Module`]
#[derive(Debug, Copy, Clone)]
pub struct GlobalIdx(pub(crate) u32);

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
    /// The initial value of the global variable.
    ///
    /// # Note
    ///
    /// This is represented by a so called initializer expression
    /// that is run at module instantiation time.
    init_expr: InitExpr,
}

impl From<wasmparser::Global<'_>> for Global {
    fn from(global: wasmparser::Global<'_>) -> Self {
        let global_type = GlobalType::from_wasmparser(global.ty);
        let init_expr = InitExpr::new(global.init_expr);
        Self {
            global_type,
            init_expr,
        }
    }
}

impl Global {
    /// Splits the [`Global`] into its global type and its global initializer.
    pub fn into_type_and_init(self) -> (GlobalType, InitExpr) {
        (self.global_type, self.init_expr)
    }
}
