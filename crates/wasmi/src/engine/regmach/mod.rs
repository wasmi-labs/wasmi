mod executor;
mod translator;

#[cfg(test)]
mod tests;

pub(crate) use self::executor::Stack;
pub(super) use self::translator::TranslationErrorInner;
pub use self::translator::{
    FuncLocalConstsIter,
    FuncTranslator,
    FuncTranslatorAllocations,
    Instr,
    TranslationError,
};
use crate::engine::CompiledFunc;
