use super::ConstExpr;
use crate::GlobalType;

#[cfg(feature = "parser")]
use super::utils::FromWasmparser;

/// The index of a global variable within a [`Module`].
///
/// [`Module`]: [`super::Module`]
#[derive(Debug, Copy, Clone)]
pub struct GlobalIdx(u32);

impl From<u32> for GlobalIdx {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl GlobalIdx {
    /// Returns the [`GlobalIdx`] as `u32`.
    pub fn into_u32(self) -> u32 {
        self.0
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
    init_expr: ConstExpr,
}

#[cfg(feature = "parser")]
impl From<wasmparser::Global<'_>> for Global {
    fn from(global: wasmparser::Global<'_>) -> Self {
        let global_type = GlobalType::from_wasmparser(global.ty);
        let init_expr = ConstExpr::new(global.init_expr);
        Self {
            global_type,
            init_expr,
        }
    }
}

impl Global {
    /// Splits the [`Global`] into its global type and its global initializer.
    pub fn into_type_and_init(self) -> (GlobalType, ConstExpr) {
        (self.global_type, self.init_expr)
    }
}
