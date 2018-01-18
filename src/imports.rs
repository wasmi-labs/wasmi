use std::collections::HashMap;
use parity_wasm::elements::{FunctionType, GlobalType, MemoryType, TableType};
use global::GlobalRef;
use memory::MemoryRef;
use func::FuncRef;
use table::TableRef;
use module::ModuleRef;
use {Error, Signature};

pub trait ImportResolver {
	fn resolve_func(
		&self,
		module_name: &str,
		field_name: &str,
		func_type: &Signature,
	) -> Result<FuncRef, Error>;

	fn resolve_global(
		&self,
		module_name: &str,
		field_name: &str,
		global_type: &GlobalType,
	) -> Result<GlobalRef, Error>;

	fn resolve_memory(
		&self,
		module_name: &str,
		field_name: &str,
		memory_type: &MemoryType,
	) -> Result<MemoryRef, Error>;

	fn resolve_table(
		&self,
		module_name: &str,
		field_name: &str,
		table_type: &TableType,
	) -> Result<TableRef, Error>;
}

pub struct ImportsBuilder<'a> {
	modules: HashMap<String, &'a ModuleImportResolver>,
}

impl<'a> Default for ImportsBuilder<'a> {
	fn default() -> Self {
		Self::new()
	}
}

impl<'a> ImportsBuilder<'a> {
	pub fn new() -> ImportsBuilder<'a> {
		ImportsBuilder { modules: HashMap::new() }
	}

	pub fn with_resolver<N: Into<String>>(
		mut self,
		name: N,
		resolver: &'a ModuleImportResolver,
	) -> Self {
		self.modules.insert(name.into(), resolver);
		self
	}

	pub fn push_resolver<N: Into<String>>(&mut self, name: N, resolver: &'a ModuleImportResolver) {
		self.modules.insert(name.into(), resolver);
	}

	pub fn resolver(&self, name: &str) -> Option<&ModuleImportResolver> {
		self.modules.get(name).cloned()
	}
}

impl<'a> ImportResolver for ImportsBuilder<'a> {
	fn resolve_func(
		&self,
		module_name: &str,
		field_name: &str,
		signature: &Signature,
	) -> Result<FuncRef, Error> {
		self.resolver(module_name).ok_or_else(||
			Error::Instantiation(format!("Module {} not found", module_name))
		)?.resolve_func(field_name, signature)
	}

	fn resolve_global(
		&self,
		module_name: &str,
		field_name: &str,
		global_type: &GlobalType,
	) -> Result<GlobalRef, Error> {
		self.resolver(module_name).ok_or_else(||
			Error::Instantiation(format!("Module {} not found", module_name))
		)?.resolve_global(field_name, global_type)
	}

	fn resolve_memory(
		&self,
		module_name: &str,
		field_name: &str,
		memory_type: &MemoryType,
	) -> Result<MemoryRef, Error> {
		self.resolver(module_name).ok_or_else(||
			Error::Instantiation(format!("Module {} not found", module_name))
		)?.resolve_memory(field_name, memory_type)
	}

	fn resolve_table(
		&self,
		module_name: &str,
		field_name: &str,
		table_type: &TableType,
	) -> Result<TableRef, Error> {
		self.resolver(module_name).ok_or_else(||
			Error::Instantiation(format!("Module {} not found", module_name))
		)?.resolve_table(field_name, table_type)
	}
}

pub trait ModuleImportResolver {
	fn resolve_func(
		&self,
		field_name: &str,
		_signature: &Signature,
	) -> Result<FuncRef, Error> {
		Err(Error::Instantiation(
			format!("Export {} not found", field_name),
		))
	}

	fn resolve_global(
		&self,
		field_name: &str,
		_global_type: &GlobalType,
	) -> Result<GlobalRef, Error> {
		Err(Error::Instantiation(
			format!("Export {} not found", field_name),
		))
	}

	fn resolve_memory(
		&self,
		field_name: &str,
		_memory_type: &MemoryType,
	) -> Result<MemoryRef, Error> {
		Err(Error::Instantiation(
			format!("Export {} not found", field_name),
		))
	}

	fn resolve_table(
		&self,
		field_name: &str,
		_table_type: &TableType,
	) -> Result<TableRef, Error> {
		Err(Error::Instantiation(
			format!("Export {} not found", field_name),
		))
	}
}

impl ModuleImportResolver for ModuleRef {
	fn resolve_func(
		&self,
		field_name: &str,
		_signature: &Signature,
	) -> Result<FuncRef, Error> {
		Ok(self.export_by_name(field_name)
			.ok_or_else(|| {
				Error::Instantiation(format!("Export {} not found", field_name))
			})?
			.as_func()
			.ok_or_else(|| {
				Error::Instantiation(format!("Export {} is not a function", field_name))
			})?)
	}

	fn resolve_global(
		&self,
		field_name: &str,
		_global_type: &GlobalType,
	) -> Result<GlobalRef, Error> {
		Ok(self.export_by_name(field_name)
			.ok_or_else(|| {
				Error::Instantiation(format!("Export {} not found", field_name))
			})?
			.as_global()
			.ok_or_else(|| {
				Error::Instantiation(format!("Export {} is not a global", field_name))
			})?)
	}

	fn resolve_memory(
		&self,
		field_name: &str,
		_memory_type: &MemoryType,
	) -> Result<MemoryRef, Error> {
		Ok(self.export_by_name(field_name)
			.ok_or_else(|| {
				Error::Instantiation(format!("Export {} not found", field_name))
			})?
			.as_memory()
			.ok_or_else(|| {
				Error::Instantiation(format!("Export {} is not a memory", field_name))
			})?)
	}

	fn resolve_table(
		&self,
		field_name: &str,
		_table_type: &TableType,
	) -> Result<TableRef, Error> {
		Ok(self.export_by_name(field_name)
			.ok_or_else(|| {
				Error::Instantiation(format!("Export {} not found", field_name))
			})?
			.as_table()
			.ok_or_else(|| {
				Error::Instantiation(format!("Export {} is not a table", field_name))
			})?)
	}
}
