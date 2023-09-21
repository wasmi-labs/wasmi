mod control_frame;
mod control_stack;
mod error;
mod inst_builder;
pub(crate) mod labels;
mod locals_registry;
mod translator;
mod value_stack;

use self::{control_frame::ControlFrame, control_stack::ControlFlowStack};
pub use self::{
    error::{TranslationError, TranslationErrorInner},
    inst_builder::{Instr, InstructionsBuilder, RelativeDepth},
    translator::{FuncTranslator, FuncTranslatorAllocations},
};
