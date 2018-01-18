use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::collections::HashMap;
use std::borrow::Cow;
use parity_wasm::elements::{External, InitExpr, Internal, Opcode, ResizableLimits, Type};
use {Error, Signature, MemoryInstance, RuntimeValue, TableInstance};
use imports::ImportResolver;
use global::{GlobalInstance, GlobalRef};
use func::{FuncRef, FuncBody, FuncInstance};
use table::TableRef;
use memory::MemoryRef;
use host::Externals;
use validation::ValidatedModule;
use common::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX};

#[derive(Clone, Debug)]
pub struct ModuleRef(Rc<ModuleInstance>);

impl ::std::ops::Deref for ModuleRef {
	type Target = ModuleInstance;
	fn deref(&self) -> &ModuleInstance {
		&self.0
	}
}

pub enum ExternVal {
	Func(FuncRef),
	Table(TableRef),
	Memory(MemoryRef),
	Global(GlobalRef),
}

impl Clone for ExternVal {
	fn clone(&self) -> Self {
		match *self {
			ExternVal::Func(ref func) => ExternVal::Func(func.clone()),
			ExternVal::Table(ref table) => ExternVal::Table(table.clone()),
			ExternVal::Memory(ref memory) => ExternVal::Memory(memory.clone()),
			ExternVal::Global(ref global) => ExternVal::Global(global.clone()),
		}
	}
}

impl fmt::Debug for ExternVal {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"ExternVal {{ {} }}",
			match *self {
				ExternVal::Func(_) => "Func",
				ExternVal::Table(_) => "Table",
				ExternVal::Memory(_) => "Memory",
				ExternVal::Global(_) => "Global",
			}
		)
	}
}

impl ExternVal {
	pub fn as_func(&self) -> Option<FuncRef> {
		match *self {
			ExternVal::Func(ref func) => Some(func.clone()),
			_ => None,
		}
	}

	pub fn as_table(&self) -> Option<TableRef> {
		match *self {
			ExternVal::Table(ref table) => Some(table.clone()),
			_ => None,
		}
	}

	pub fn as_memory(&self) -> Option<MemoryRef> {
		match *self {
			ExternVal::Memory(ref memory) => Some(memory.clone()),
			_ => None,
		}
	}

	pub fn as_global(&self) -> Option<GlobalRef> {
		match *self {
			ExternVal::Global(ref global) => Some(global.clone()),
			_ => None,
		}
	}
}

#[derive(Debug)]
pub struct ModuleInstance {
	signatures: RefCell<Vec<Rc<Signature>>>,
	tables: RefCell<Vec<TableRef>>,
	funcs: RefCell<Vec<FuncRef>>,
	memories: RefCell<Vec<MemoryRef>>,
	globals: RefCell<Vec<GlobalRef>>,
	exports: RefCell<HashMap<String, ExternVal>>,
}

impl ModuleInstance {
	fn default() -> Self {
		ModuleInstance {
			funcs: RefCell::new(Vec::new()),
			signatures: RefCell::new(Vec::new()),
			tables: RefCell::new(Vec::new()),
			memories: RefCell::new(Vec::new()),
			globals: RefCell::new(Vec::new()),
			exports: RefCell::new(HashMap::new()),
		}
	}

	pub(crate) fn memory_by_index(&self, idx: u32) -> Option<MemoryRef> {
		self.memories.borrow_mut().get(idx as usize).cloned()
	}

	pub(crate) fn table_by_index(&self, idx: u32) -> Option<TableRef> {
		self.tables.borrow_mut().get(idx as usize).cloned()
	}

	pub(crate) fn global_by_index(&self, idx: u32) -> Option<GlobalRef> {
		self.globals.borrow_mut().get(idx as usize).cloned()
	}

	pub(crate) fn func_by_index(&self, idx: u32) -> Option<FuncRef> {
		self.funcs.borrow().get(idx as usize).cloned()
	}

	pub fn export_by_name(&self, name: &str) -> Option<ExternVal> {
		self.exports.borrow().get(name).cloned()
	}

	pub(crate) fn signature_by_index(&self, idx: u32) -> Option<Rc<Signature>> {
		self.signatures.borrow().get(idx as usize).cloned()
	}

	fn push_func(&self, func: FuncRef) {
		self.funcs.borrow_mut().push(func);
	}

	fn push_signature(&self, signature: Rc<Signature>) {
		self.signatures.borrow_mut().push(signature)
	}

	fn push_memory(&self, memory: MemoryRef) {
		self.memories.borrow_mut().push(memory)
	}

	fn push_table(&self, table: TableRef) {
		self.tables.borrow_mut().push(table)
	}

	fn push_global(&self, global: GlobalRef) {
		self.globals.borrow_mut().push(global)
	}

	fn insert_export<N: Into<String>>(&self, name: N, extern_val: ExternVal) {
		self.exports.borrow_mut().insert(name.into(), extern_val);
	}

	fn alloc_module(
		validated_module: &ValidatedModule,
		extern_vals: &[ExternVal]
	) -> Result<ModuleRef, Error> {
		let module = validated_module.module();
		let instance = ModuleRef(Rc::new(ModuleInstance::default()));

		for &Type::Function(ref ty) in module.type_section().map(|ts| ts.types()).unwrap_or(&[]) {
			let signature = Rc::new(Signature::from_elements(ty.clone()));
			instance.push_signature(signature);
		}

		{
			let imports = module.import_section().map(|is| is.entries()).unwrap_or(
				&[],
			);
			if imports.len() != extern_vals.len() {
				return Err(Error::Instantiation(format!(
					"extern_vals length is not equal to import section entries"
				)));
			}

			for (import, extern_val) in
				Iterator::zip(imports.into_iter(), extern_vals.into_iter())
			{
				match (import.external(), extern_val) {
					(&External::Function(fn_type_idx), &ExternVal::Func(ref func)) => {
						let expected_fn_type = instance.signature_by_index(fn_type_idx).expect(
							"Due to validation function type should exists",
						);
						let actual_fn_type = func.signature();
						if &*expected_fn_type != actual_fn_type {
							return Err(Error::Instantiation(format!(
								"Expected function with type {:?}, but actual type is {:?} for entry {}",
								expected_fn_type,
								actual_fn_type,
								import.field(),
							)));
						}
						instance.push_func(func.clone())
					}
					(&External::Table(ref tt), &ExternVal::Table(ref table)) => {
						match_limits(table.limits(), tt.limits())?;
						instance.push_table(table.clone());
					}
					(&External::Memory(ref mt), &ExternVal::Memory(ref memory)) => {
						match_limits(memory.limits(), mt.limits())?;
						instance.push_memory(memory.clone());
					}
					(&External::Global(ref gl), &ExternVal::Global(ref global)) => {
						if gl.content_type() != global.elements_value_type() {
							return Err(Error::Instantiation(format!(
								"Expect global with {:?} type, but provided global with {:?} type",
								gl.content_type(),
								global.value_type(),
							)));
						}
						instance.push_global(global.clone());
					}
					(expected_import, actual_extern_val) => {
						return Err(Error::Instantiation(format!(
							"Expected {:?} type, but provided {:?} extern_val",
							expected_import,
							actual_extern_val
						)));
					}
				}
			}
		}

		let labels = validated_module.labels();
		{
			let funcs = module.function_section().map(|fs| fs.entries()).unwrap_or(
				&[],
			);
			let bodies = module.code_section().map(|cs| cs.bodies()).unwrap_or(&[]);
			debug_assert!(
				funcs.len() == bodies.len(),
				"Due to validation func and body counts must match"
			);

			for (index, (ty, body)) in
				Iterator::zip(funcs.into_iter(), bodies.into_iter()).enumerate()
			{
				let signature = instance.signature_by_index(ty.type_ref()).expect(
					"Due to validation type should exists",
				);
				let labels = labels.get(&index).expect(
					"At func validation time labels are collected; Collected labels are added by index; qed",
				).clone();
				let func_body = FuncBody {
					locals: body.locals().to_vec(),
					opcodes: body.code().clone(),
					labels: labels,
				};
				let func_instance =
					FuncInstance::alloc_internal(instance.clone(), signature, func_body);
				instance.push_func(func_instance);
			}
		}

		for table_type in module.table_section().map(|ts| ts.entries()).unwrap_or(&[]) {
			let table = TableInstance::alloc(
				table_type.limits().initial(),
				table_type.limits().maximum(),
			)?;
			instance.push_table(table);
		}

		for memory_type in module.memory_section().map(|ms| ms.entries()).unwrap_or(
			&[],
		)
		{
			let memory = MemoryInstance::alloc(
				memory_type.limits().initial(),
				memory_type.limits().maximum()
			)?;
			instance.push_memory(memory);
		}

		for global_entry in module.global_section().map(|gs| gs.entries()).unwrap_or(
			&[],
		)
		{
			let init_val = eval_init_expr(global_entry.init_expr(), &*instance);
			let global = GlobalInstance::alloc(
				init_val,
				global_entry.global_type().is_mutable(),
			);
			instance.push_global(global);
		}

		for export in module.export_section().map(|es| es.entries()).unwrap_or(
			&[],
		)
		{
			let field = export.field();
			let extern_val: ExternVal = match *export.internal() {
				Internal::Function(idx) => {
					let func = instance.func_by_index(idx).expect(
						"Due to validation func should exists",
					);
					ExternVal::Func(func)
				}
				Internal::Global(idx) => {
					let global = instance.global_by_index(idx).expect(
						"Due to validation global should exists",
					);
					ExternVal::Global(global)
				}
				Internal::Memory(idx) => {
					let memory = instance.memory_by_index(idx).expect(
						"Due to validation memory should exists",
					);
					ExternVal::Memory(memory)
				}
				Internal::Table(idx) => {
					let table = instance.table_by_index(idx).expect(
						"Due to validation table should exists",
					);
					ExternVal::Table(table)
				}
			};
			instance.insert_export(field, extern_val);
		}

		Ok(instance)
	}

	fn instantiate_with_externvals(
		validated_module: &ValidatedModule,
		extern_vals: &[ExternVal],
	) -> Result<ModuleRef, Error> {
		let module = validated_module.module();

		let module_ref = ModuleInstance::alloc_module(validated_module, extern_vals)?;

		for element_segment in module.elements_section().map(|es| es.entries()).unwrap_or(
			&[],
		)
		{
			let offset_val = match eval_init_expr(element_segment.offset(), &module_ref) {
				RuntimeValue::I32(v) => v as u32,
				_ => panic!("Due to validation elem segment offset should evaluate to i32"),
			};

			let table_inst = module_ref.table_by_index(DEFAULT_TABLE_INDEX).expect(
				"Due to validation default table should exists",
			);
			for (j, func_idx) in element_segment.members().into_iter().enumerate() {
				let func = module_ref.func_by_index(*func_idx).expect(
					"Due to validation funcs from element segments should exists",
				);

				table_inst.set(offset_val + j as u32, Some(func))?;
			}
		}

		for data_segment in module.data_section().map(|ds| ds.entries()).unwrap_or(&[]) {
			let offset_val = match eval_init_expr(data_segment.offset(), &module_ref) {
				RuntimeValue::I32(v) => v as u32,
				_ => panic!("Due to validation data segment offset should evaluate to i32"),
			};

			let memory_inst = module_ref.memory_by_index(DEFAULT_MEMORY_INDEX).expect(
				"Due to validation default memory should exists",
			);
			memory_inst.set(offset_val, data_segment.value())?;
		}

		Ok(module_ref)
	}

	pub fn new<'m, I: ImportResolver>(
		validated_module: &'m ValidatedModule,
		imports: &I,
	) -> Result<NotStartedModuleRef<'m>, Error> {
		let module = validated_module.module();

		let mut extern_vals = Vec::new();
		for import_entry in module.import_section().map(|s| s.entries()).unwrap_or(&[]) {
			let module_name = import_entry.module();
			let field_name = import_entry.field();
			let extern_val = match *import_entry.external() {
				External::Function(fn_ty_idx) => {
					let types = module.type_section().map(|s| s.types()).unwrap_or(&[]);
					let &Type::Function(ref func_type) = types
						.get(fn_ty_idx as usize)
						.expect("Due to validation functions should have valid types");
					let signature = Signature::from_elements(func_type.clone());
					let func = imports.resolve_func(module_name, field_name, &signature)?;
					ExternVal::Func(func)
				}
				External::Table(ref table_type) => {
					let table = imports.resolve_table(module_name, field_name, table_type)?;
					ExternVal::Table(table)
				}
				External::Memory(ref memory_type) => {
					let memory = imports.resolve_memory(module_name, field_name, memory_type)?;
					ExternVal::Memory(memory)
				}
				External::Global(ref global_type) => {
					let global = imports.resolve_global(module_name, field_name, global_type)?;
					ExternVal::Global(global)
				}
			};
			extern_vals.push(extern_val);
		}

		let instance = Self::instantiate_with_externvals(validated_module, &extern_vals)?;
		Ok(NotStartedModuleRef {
			validated_module,
			instance,
		})
	}

	pub fn invoke_export<E: Externals>(
		&self,
		func_name: &str,
		args: &[RuntimeValue],
		externals: &mut E,
	) -> Result<Option<RuntimeValue>, Error> {
		let extern_val = self.export_by_name(func_name).ok_or_else(|| {
			Error::Function(format!("Module doesn't have export {}", func_name))
		})?;

		let func_instance = match extern_val {
			ExternVal::Func(func_instance) => func_instance,
			unexpected => {
				return Err(Error::Function(format!(
					"Export {} is not a function, but {:?}",
					func_name,
					unexpected
				)));
			}
		};

		FuncInstance::invoke(func_instance.clone(), Cow::Borrowed(args), externals)
	}
}

pub struct NotStartedModuleRef<'a> {
	validated_module: &'a ValidatedModule,
	instance: ModuleRef,
}

impl<'a> NotStartedModuleRef<'a> {
	pub fn not_started_instance(&self) -> &ModuleRef {
		&self.instance
	}

	pub fn run_start<'b, E: Externals>(self, state: &'b mut E) -> Result<ModuleRef, Error> {
		if let Some(start_fn_idx) = self.validated_module.module().start_section() {
			let start_func = self.instance.func_by_index(start_fn_idx).expect(
				"Due to validation start function should exists",
			);
			FuncInstance::invoke(start_func, Cow::Borrowed(&[]), state)?;
		}
		Ok(self.instance)
	}

	pub fn assert_no_start(self) -> ModuleRef {
		assert!(self.validated_module.module().start_section().is_none());
		self.instance
	}
}

fn eval_init_expr(init_expr: &InitExpr, module: &ModuleInstance) -> RuntimeValue {
	let code = init_expr.code();
	debug_assert!(
		code.len() == 2,
		"Due to validation `code`.len() should be 2"
	);
	match code[0] {
		Opcode::I32Const(v) => v.into(),
		Opcode::I64Const(v) => v.into(),
		Opcode::F32Const(v) => RuntimeValue::decode_f32(v),
		Opcode::F64Const(v) => RuntimeValue::decode_f64(v),
		Opcode::GetGlobal(idx) => {
			let global = module.global_by_index(idx).expect(
				"Due to validation global should exists in module",
			);
			global.get()
		}
		_ => panic!("Due to validation init should be a const expr"),
	}
}

fn match_limits(l1: &ResizableLimits, l2: &ResizableLimits) -> Result<(), Error> {
	if l1.initial() < l2.initial() {
		return Err(Error::Instantiation(format!(
			"trying to import with limits l1.initial={} and l2.initial={}",
			l1.initial(),
			l2.initial()
		)));
	}

	match (l1.maximum(), l2.maximum()) {
		(_, None) => (),
		(Some(m1), Some(m2)) if m1 <= m2 => (),
		_ => {
			return Err(Error::Instantiation(format!(
				"trying to import with limits l1.max={:?} and l2.max={:?}",
				l1.maximum(),
				l2.maximum()
			)))
		}
	}

	Ok(())
}

pub fn check_limits(limits: &ResizableLimits) -> Result<(), Error> {
	if let Some(maximum) = limits.maximum() {
		if maximum < limits.initial() {
			return Err(Error::Instantiation(format!(
				"maximum limit {} is less than minimum {}",
				maximum,
				limits.initial()
			)));
		}
	}

	Ok(())
}
