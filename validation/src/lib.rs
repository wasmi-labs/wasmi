#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

pub mod stack;

/// Index of default linear memory.
pub const DEFAULT_MEMORY_INDEX: u32 = 0;
/// Index of default table.
pub const DEFAULT_TABLE_INDEX: u32 = 0;

/// Maximal number of pages that a wasm instance supports.
pub const LINEAR_MEMORY_MAX_PAGES: u32 = 65536;

use alloc::{string::String, vec::Vec};
use core::fmt;
#[cfg(feature = "std")]
use std::error;

use self::context::ModuleContextBuilder;
use parity_wasm::elements::{
    BlockType,
    ExportEntry,
    External,
    FuncBody,
    GlobalEntry,
    GlobalType,
    InitExpr,
    Instruction,
    Internal,
    MemoryType,
    Module,
    ResizableLimits,
    TableType,
    Type,
    ValueType,
};

pub mod context;
pub mod func;
pub mod util;

#[cfg(test)]
mod tests;

// TODO: Consider using a type other than String, because
// of formatting machinary is not welcomed in substrate runtimes.
#[derive(Debug)]
pub struct Error(pub String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "std")]
impl error::Error for Error {
    fn description(&self) -> &str {
        &self.0
    }
}

impl From<stack::Error> for Error {
    fn from(e: stack::Error) -> Error {
        Error(format!("Stack: {}", e))
    }
}

pub trait Validator {
    /// Custom inputs to the validator constructor.
    type Input;
    type Output;
    type FuncValidator: FuncValidator;

    fn new(module: &Module, input: Self::Input) -> Self;

    fn func_validator_input(&mut self) -> <Self::FuncValidator as FuncValidator>::Input;

    fn on_function_validated(
        &mut self,
        index: u32,
        output: <<Self as Validator>::FuncValidator as FuncValidator>::Output,
    );

    fn finish(self) -> Self::Output;
}

pub trait FuncValidator {
    /// Custom inputs to the function validator constructor.
    type Input;

    type Output;

    fn new(ctx: &func::FunctionValidationContext, body: &FuncBody, input: Self::Input) -> Self;

    fn next_instruction(
        &mut self,
        ctx: &mut func::FunctionValidationContext,
        instruction: &Instruction,
    ) -> Result<(), Error>;

    fn finish(self, ctx: &func::FunctionValidationContext) -> Self::Output;
}

/// A module validator that just validates modules and produces no result.
pub struct PlainValidator;

impl Validator for PlainValidator {
    type Input = ();
    type Output = ();
    type FuncValidator = PlainFuncValidator;
    fn new(_module: &Module, _args: Self::Input) -> PlainValidator {
        PlainValidator
    }
    fn func_validator_input(&mut self) -> <Self::FuncValidator as FuncValidator>::Input {}
    fn on_function_validated(
        &mut self,
        _index: u32,
        _output: <<Self as Validator>::FuncValidator as FuncValidator>::Output,
    ) {
    }
    fn finish(self) {}
}

/// A function validator that just validates modules and produces no result.
pub struct PlainFuncValidator;

impl FuncValidator for PlainFuncValidator {
    type Input = ();
    type Output = ();

    fn new(
        _ctx: &func::FunctionValidationContext,
        _body: &FuncBody,
        _input: Self::Input,
    ) -> PlainFuncValidator {
        PlainFuncValidator
    }

    fn next_instruction(
        &mut self,
        ctx: &mut func::FunctionValidationContext,
        instruction: &Instruction,
    ) -> Result<(), Error> {
        ctx.step(instruction)
    }

    fn finish(self, _ctx: &func::FunctionValidationContext) {}
}

pub fn validate_module<V: Validator>(
    module: &Module,
    input: <V as Validator>::Input,
) -> Result<V::Output, Error> {
    let mut context_builder = ModuleContextBuilder::new();
    let mut imported_globals = Vec::new();
    let mut validation = V::new(module, input);

    // Copy types from module as is.
    context_builder.set_types(
        module
            .type_section()
            .map(|ts| {
                ts.types()
                    .iter()
                    .map(|&Type::Function(ref ty)| ty)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default(),
    );

    // Fill elements with imported values.
    for import_entry in module
        .import_section()
        .map(|i| i.entries())
        .unwrap_or_default()
    {
        match *import_entry.external() {
            External::Function(idx) => context_builder.push_func_type_index(idx),
            External::Table(ref table) => context_builder.push_table(*table),
            External::Memory(ref memory) => context_builder.push_memory(*memory),
            External::Global(ref global) => {
                context_builder.push_global(*global);
                imported_globals.push(*global);
            }
        }
    }

    // Concatenate elements with defined in the module.
    if let Some(function_section) = module.function_section() {
        for func_entry in function_section.entries() {
            context_builder.push_func_type_index(func_entry.type_ref())
        }
    }
    if let Some(table_section) = module.table_section() {
        for table_entry in table_section.entries() {
            validate_table_type(table_entry)?;
            context_builder.push_table(*table_entry);
        }
    }
    if let Some(mem_section) = module.memory_section() {
        for mem_entry in mem_section.entries() {
            validate_memory_type(mem_entry)?;
            context_builder.push_memory(*mem_entry);
        }
    }
    if let Some(global_section) = module.global_section() {
        for global_entry in global_section.entries() {
            validate_global_entry(global_entry, &imported_globals)?;
            context_builder.push_global(*global_entry.global_type());
        }
    }

    let context = context_builder.build();

    let function_section_len = module
        .function_section()
        .map(|s| s.entries().len())
        .unwrap_or(0);
    let code_section_len = module.code_section().map(|s| s.bodies().len()).unwrap_or(0);
    if function_section_len != code_section_len {
        return Err(Error(format!(
            "length of function section is {}, while len of code section is {}",
            function_section_len, code_section_len
        )));
    }

    // validate every function body in user modules
    if function_section_len != 0 {
        // tests use invalid code
        let function_section = module
            .function_section()
            .expect("function_section_len != 0; qed");
        let code_section = module
            .code_section()
            .expect("function_section_len != 0; function_section_len == code_section_len; qed");
        // check every function body
        for (index, function) in function_section.entries().iter().enumerate() {
            let function_body = code_section
                .bodies()
                .get(index as usize)
                .ok_or_else(|| Error(format!("Missing body for function {}", index)))?;
            let func_validator_input = validation.func_validator_input();
            let output = func::drive::<V::FuncValidator>(
                &context,
                function,
                function_body,
                func_validator_input,
            )
            .map_err(|Error(ref msg)| {
                Error(format!(
                    "Function #{} reading/validation error: {}",
                    index, msg
                ))
            })?;
            validation.on_function_validated(index as u32, output);
        }
    }

    // validate start section
    if let Some(start_fn_idx) = module.start_section() {
        let (params, return_ty) = context.require_function(start_fn_idx)?;
        if return_ty != BlockType::NoResult || !params.is_empty() {
            return Err(Error(
                "start function expected to have type [] -> []".into(),
            ));
        }
    }

    // validate export section
    if let Some(export_section) = module.export_section() {
        let mut export_names = export_section
            .entries()
            .iter()
            .map(ExportEntry::field)
            .collect::<Vec<_>>();

        export_names.sort_unstable();

        for (fst, snd) in export_names.iter().zip(export_names.iter().skip(1)) {
            if fst == snd {
                return Err(Error(format!("duplicate export {}", fst)));
            }
        }

        for export in export_section.entries() {
            match *export.internal() {
                Internal::Function(function_index) => {
                    context.require_function(function_index)?;
                }
                Internal::Global(global_index) => {
                    context.require_global(global_index, None)?;
                }
                Internal::Memory(memory_index) => {
                    context.require_memory(memory_index)?;
                }
                Internal::Table(table_index) => {
                    context.require_table(table_index)?;
                }
            }
        }
    }

    // validate import section
    if let Some(import_section) = module.import_section() {
        for import in import_section.entries() {
            match *import.external() {
                External::Function(function_type_index) => {
                    context.require_function_type(function_type_index)?;
                }
                External::Global(_) => {}
                External::Memory(ref memory_type) => {
                    validate_memory_type(memory_type)?;
                }
                External::Table(ref table_type) => {
                    validate_table_type(table_type)?;
                }
            }
        }
    }

    // there must be no greater than 1 table in tables index space
    if context.tables().len() > 1 {
        return Err(Error(format!(
            "too many tables in index space: {}",
            context.tables().len()
        )));
    }

    // there must be no greater than 1 linear memory in memory index space
    if context.memories().len() > 1 {
        return Err(Error(format!(
            "too many memory regions in index space: {}",
            context.memories().len()
        )));
    }

    // use data section to initialize linear memory regions
    if let Some(data_section) = module.data_section() {
        for data_segment in data_section.entries() {
            context.require_memory(data_segment.index())?;
            let offset = data_segment
                .offset()
                .as_ref()
                .ok_or_else(|| Error("passive memory segments are not supported".into()))?;
            let init_ty = expr_const_type(offset, context.globals())?;
            if init_ty != ValueType::I32 {
                return Err(Error("segment offset should return I32".into()));
            }
        }
    }

    // use element section to fill tables
    if let Some(element_section) = module.elements_section() {
        for element_segment in element_section.entries() {
            context.require_table(element_segment.index())?;
            let offset = element_segment
                .offset()
                .as_ref()
                .ok_or_else(|| Error("passive element segments are not supported".into()))?;
            let init_ty = expr_const_type(offset, context.globals())?;
            if init_ty != ValueType::I32 {
                return Err(Error("segment offset should return I32".into()));
            }

            for function_index in element_segment.members() {
                context.require_function(*function_index)?;
            }
        }
    }

    Ok(validation.finish())
}

fn validate_limits(limits: &ResizableLimits) -> Result<(), Error> {
    if let Some(maximum) = limits.maximum() {
        if limits.initial() > maximum {
            return Err(Error(format!(
                "maximum limit {} is less than minimum {}",
                maximum,
                limits.initial()
            )));
        }
    }
    Ok(())
}

fn validate_memory_type(memory_type: &MemoryType) -> Result<(), Error> {
    let initial = memory_type.limits().initial();
    let maximum: Option<u32> = memory_type.limits().maximum();
    validate_memory(initial, maximum).map_err(Error)
}

pub fn validate_memory(initial: u32, maximum: Option<u32>) -> Result<(), String> {
    if initial > LINEAR_MEMORY_MAX_PAGES {
        return Err(format!(
            "initial memory size must be at most {} pages",
            LINEAR_MEMORY_MAX_PAGES
        ));
    }
    if let Some(maximum) = maximum {
        if initial > maximum {
            return Err(format!(
                "maximum limit {} is less than minimum {}",
                maximum, initial,
            ));
        }

        if maximum > LINEAR_MEMORY_MAX_PAGES {
            return Err(format!(
                "maximum memory size must be at most {} pages",
                LINEAR_MEMORY_MAX_PAGES
            ));
        }
    }
    Ok(())
}

fn validate_table_type(table_type: &TableType) -> Result<(), Error> {
    validate_limits(table_type.limits())
}

fn validate_global_entry(global_entry: &GlobalEntry, globals: &[GlobalType]) -> Result<(), Error> {
    let init = global_entry.init_expr();
    let init_expr_ty = expr_const_type(init, globals)?;
    if init_expr_ty != global_entry.global_type().content_type() {
        return Err(Error(format!(
            "Trying to initialize variable of type {:?} with value of type {:?}",
            global_entry.global_type().content_type(),
            init_expr_ty
        )));
    }
    Ok(())
}

/// Returns type of this constant expression.
fn expr_const_type(init_expr: &InitExpr, globals: &[GlobalType]) -> Result<ValueType, Error> {
    let code = init_expr.code();
    if code.len() != 2 {
        return Err(Error(
            "Init expression should always be with length 2".into(),
        ));
    }
    let expr_ty: ValueType = match code[0] {
        Instruction::I32Const(_) => ValueType::I32,
        Instruction::I64Const(_) => ValueType::I64,
        Instruction::F32Const(_) => ValueType::F32,
        Instruction::F64Const(_) => ValueType::F64,
        Instruction::GetGlobal(idx) => match globals.get(idx as usize) {
            Some(target_global) => {
                if target_global.is_mutable() {
                    return Err(Error(format!("Global {} is mutable", idx)));
                }
                target_global.content_type()
            }
            None => {
                return Err(Error(format!(
                    "Global {} doesn't exists or not yet defined",
                    idx
                )));
            }
        },
        _ => return Err(Error("Non constant opcode in init expr".into())),
    };
    if code[1] != Instruction::End {
        return Err(Error("Expression doesn't ends with `end` opcode".into()));
    }
    Ok(expr_ty)
}
