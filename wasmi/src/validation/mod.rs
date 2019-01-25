#[allow(unused_imports)]
use alloc::prelude::*;
use core::fmt;
#[cfg(feature = "std")]
use std::error;

#[cfg(not(feature = "std"))]
use hashbrown::HashSet;
#[cfg(feature = "std")]
use std::collections::HashSet;

use self::context::ModuleContextBuilder;
use self::func::FunctionReader;
use common::stack;
use isa;
use memory_units::Pages;
use parity_wasm::elements::{
    BlockType, External, GlobalEntry, GlobalType, InitExpr, Instruction, Internal, MemoryType,
    Module, ResizableLimits, TableType, Type, ValueType,
};

mod context;
mod func;
mod util;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Error(String);

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

#[derive(Clone)]
pub struct ValidatedModule {
    pub code_map: Vec<isa::Instructions>,
    pub module: Module,
}

impl ::core::ops::Deref for ValidatedModule {
    type Target = Module;
    fn deref(&self) -> &Module {
        &self.module
    }
}

pub fn deny_floating_point(module: &Module) -> Result<(), Error> {
    if let Some(code) = module.code_section() {
        for op in code.bodies().iter().flat_map(|body| body.code().elements()) {
            use parity_wasm::elements::Instruction::*;

            macro_rules! match_eq {
                ($pattern:pat) => {
                    |val| if let $pattern = *val { true } else { false }
                };
            }

            const DENIED: &[fn(&Instruction) -> bool] = &[
                match_eq!(F32Load(_, _)),
                match_eq!(F64Load(_, _)),
                match_eq!(F32Store(_, _)),
                match_eq!(F64Store(_, _)),
                match_eq!(F32Const(_)),
                match_eq!(F64Const(_)),
                match_eq!(F32Eq),
                match_eq!(F32Ne),
                match_eq!(F32Lt),
                match_eq!(F32Gt),
                match_eq!(F32Le),
                match_eq!(F32Ge),
                match_eq!(F64Eq),
                match_eq!(F64Ne),
                match_eq!(F64Lt),
                match_eq!(F64Gt),
                match_eq!(F64Le),
                match_eq!(F64Ge),
                match_eq!(F32Abs),
                match_eq!(F32Neg),
                match_eq!(F32Ceil),
                match_eq!(F32Floor),
                match_eq!(F32Trunc),
                match_eq!(F32Nearest),
                match_eq!(F32Sqrt),
                match_eq!(F32Add),
                match_eq!(F32Sub),
                match_eq!(F32Mul),
                match_eq!(F32Div),
                match_eq!(F32Min),
                match_eq!(F32Max),
                match_eq!(F32Copysign),
                match_eq!(F64Abs),
                match_eq!(F64Neg),
                match_eq!(F64Ceil),
                match_eq!(F64Floor),
                match_eq!(F64Trunc),
                match_eq!(F64Nearest),
                match_eq!(F64Sqrt),
                match_eq!(F64Add),
                match_eq!(F64Sub),
                match_eq!(F64Mul),
                match_eq!(F64Div),
                match_eq!(F64Min),
                match_eq!(F64Max),
                match_eq!(F64Copysign),
                match_eq!(F32ConvertSI32),
                match_eq!(F32ConvertUI32),
                match_eq!(F32ConvertSI64),
                match_eq!(F32ConvertUI64),
                match_eq!(F32DemoteF64),
                match_eq!(F64ConvertSI32),
                match_eq!(F64ConvertUI32),
                match_eq!(F64ConvertSI64),
                match_eq!(F64ConvertUI64),
                match_eq!(F64PromoteF32),
                match_eq!(F32ReinterpretI32),
                match_eq!(F64ReinterpretI64),
                match_eq!(I32TruncSF32),
                match_eq!(I32TruncUF32),
                match_eq!(I32TruncSF64),
                match_eq!(I32TruncUF64),
                match_eq!(I64TruncSF32),
                match_eq!(I64TruncUF32),
                match_eq!(I64TruncSF64),
                match_eq!(I64TruncUF64),
                match_eq!(I32ReinterpretF32),
                match_eq!(I64ReinterpretF64),
            ];

            if DENIED.iter().any(|is_denied| is_denied(op)) {
                return Err(Error(format!("Floating point operation denied: {:?}", op)));
            }
        }
    }

    if let (Some(sec), Some(types)) = (module.function_section(), module.type_section()) {
        use parity_wasm::elements::{Type, ValueType};

        let types = types.types();

        for sig in sec.entries() {
            if let Some(typ) = types.get(sig.type_ref() as usize) {
                match *typ {
                    Type::Function(ref func) => {
                        if func
                            .params()
                            .iter()
                            .chain(func.return_type().as_ref())
                            .any(|&typ| typ == ValueType::F32 || typ == ValueType::F64)
                        {
                            return Err(Error(format!("Use of floating point types denied")));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn validate_module(module: Module) -> Result<ValidatedModule, Error> {
    let mut context_builder = ModuleContextBuilder::new();
    let mut imported_globals = Vec::new();
    let mut code_map = Vec::new();

    // Copy types from module as is.
    context_builder.set_types(
        module
            .type_section()
            .map(|ts| {
                ts.types()
                    .into_iter()
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
            External::Table(ref table) => context_builder.push_table(table.clone()),
            External::Memory(ref memory) => context_builder.push_memory(memory.clone()),
            External::Global(ref global) => {
                context_builder.push_global(global.clone());
                imported_globals.push(global.clone());
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
            context_builder.push_table(table_entry.clone());
        }
    }
    if let Some(mem_section) = module.memory_section() {
        for mem_entry in mem_section.entries() {
            validate_memory_type(mem_entry)?;
            context_builder.push_memory(mem_entry.clone());
        }
    }
    if let Some(global_section) = module.global_section() {
        for global_entry in global_section.entries() {
            validate_global_entry(global_entry, &imported_globals)?;
            context_builder.push_global(global_entry.global_type().clone());
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
                .ok_or(Error(format!("Missing body for function {}", index)))?;
            let code =
                FunctionReader::read_function(&context, function, function_body).map_err(|e| {
                    let Error(ref msg) = e;
                    Error(format!(
                        "Function #{} reading/validation error: {}",
                        index, msg
                    ))
                })?;
            code_map.push(code);
        }
    }

    // validate start section
    if let Some(start_fn_idx) = module.start_section() {
        let (params, return_ty) = context.require_function(start_fn_idx)?;
        if return_ty != BlockType::NoResult || params.len() != 0 {
            return Err(Error(
                "start function expected to have type [] -> []".into(),
            ));
        }
    }

    // validate export section
    if let Some(export_section) = module.export_section() {
        let mut export_names = HashSet::with_capacity(export_section.entries().len());
        for export in export_section.entries() {
            // HashSet::insert returns false if item already in set.
            let duplicate = export_names.insert(export.field()) == false;
            if duplicate {
                return Err(Error(format!("duplicate export {}", export.field())));
            }
            match *export.internal() {
                Internal::Function(function_index) => {
                    context.require_function(function_index)?;
                }
                Internal::Global(global_index) => {
                    context.require_global(global_index, Some(false))?;
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
                External::Global(ref global_type) => {
                    if global_type.is_mutable() {
                        return Err(Error(format!(
                            "trying to import mutable global {}",
                            import.field()
                        )));
                    }
                }
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
            let init_ty = expr_const_type(data_segment.offset(), context.globals())?;
            if init_ty != ValueType::I32 {
                return Err(Error("segment offset should return I32".into()));
            }
        }
    }

    // use element section to fill tables
    if let Some(element_section) = module.elements_section() {
        for element_segment in element_section.entries() {
            context.require_table(element_segment.index())?;

            let init_ty = expr_const_type(element_segment.offset(), context.globals())?;
            if init_ty != ValueType::I32 {
                return Err(Error("segment offset should return I32".into()));
            }

            for function_index in element_segment.members() {
                context.require_function(*function_index)?;
            }
        }
    }

    Ok(ValidatedModule { module, code_map })
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
    let initial: Pages = Pages(memory_type.limits().initial() as usize);
    let maximum: Option<Pages> = memory_type.limits().maximum().map(|m| Pages(m as usize));
    ::memory::validate_memory(initial, maximum).map_err(Error)
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
