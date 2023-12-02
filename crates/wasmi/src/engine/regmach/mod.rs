mod translator;

#[cfg(test)]
mod tests;

pub(super) use self::translator::TranslationErrorInner;
pub use self::translator::{
    FuncLocalConstsIter,
    FuncTranslator,
    FuncTranslatorAllocations,
    Instr,
    TranslationError,
};
use crate::engine::CompiledFunc;
