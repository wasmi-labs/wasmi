use std::error;
use std::fmt;
use std::collections::{HashMap, HashSet};
use parity_wasm::elements::{
	BlockType, External, GlobalEntry, GlobalType, Internal, MemoryType, Module, Opcode,
	ResizableLimits, TableType, ValueType, InitExpr, Type
};
use common::stack;
use self::context::ModuleContextBuilder;
use self::func::Validator;
use memory_units::Pages;

mod context;
mod func;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Error(String);

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

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
	pub labels: HashMap<usize, HashMap<usize, usize>>,
	pub module: Module,
}

impl ::std::ops::Deref for ValidatedModule {
	type Target = Module;
	fn deref(&self) -> &Module {
		&self.module
	}
}

pub fn validate_module(module: Module) -> Result<ValidatedModule, Error> {
	let mut context_builder = ModuleContextBuilder::new();
	let mut imported_globals = Vec::new();
	let mut labels = HashMap::new();

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
			function_section_len,
			code_section_len
		)));
	}

	// validate every function body in user modules
	if function_section_len != 0 {
		// tests use invalid code
		let function_section = module.function_section().expect(
			"function_section_len != 0; qed",
		);
		let code_section = module.code_section().expect(
			"function_section_len != 0; function_section_len == code_section_len; qed",
		);
		// check every function body
		for (index, function) in function_section.entries().iter().enumerate() {
			let function_body = code_section.bodies().get(index as usize).ok_or(
				Error(format!(
					"Missing body for function {}",
					index
				)),
			)?;
			let func_labels = Validator::validate_function(&context, function, function_body)
				.map_err(|e| {
					let Error(ref msg) = e;
					Error(format!("Function #{} validation error: {}", index, msg))
				})?;
			labels.insert(index, func_labels);
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
				return Err(Error(
					format!("duplicate export {}", export.field()),
				));
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

	Ok(ValidatedModule {
		module,
		labels
	})
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
		Opcode::I32Const(_) => ValueType::I32,
		Opcode::I64Const(_) => ValueType::I64,
		Opcode::F32Const(_) => ValueType::F32,
		Opcode::F64Const(_) => ValueType::F64,
		Opcode::GetGlobal(idx) => {
			match globals.get(idx as usize) {
				Some(target_global) => {
					if target_global.is_mutable() {
						return Err(Error(format!("Global {} is mutable", idx)));
					}
					target_global.content_type()
				}
				None => {
					return Err(Error(
						format!("Global {} doesn't exists or not yet defined", idx),
					))
				}
			}
		}
		_ => return Err(Error("Non constant opcode in init expr".into())),
	};
	if code[1] != Opcode::End {
		return Err(Error("Expression doesn't ends with `end` opcode".into()));
	}
	Ok(expr_ty)
}
