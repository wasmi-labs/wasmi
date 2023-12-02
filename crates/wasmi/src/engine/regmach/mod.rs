mod translator;

pub(super) use self::translator::TranslationErrorInner;
pub use self::translator::{
    FuncLocalConstsIter,
    FuncTranslator,
    FuncTranslatorAllocations,
    Instr,
    TranslationError,
};
use crate::engine::CompiledFunc;
