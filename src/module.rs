use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::collections::HashMap;
use parity_wasm::elements::{External, InitExpr, Internal, Opcode, ResizableLimits, Type};
use {LoadedModule, Error, Signature, MemoryInstance, RuntimeValue, TableInstance};
use imports::ImportResolver;
use global::{GlobalInstance, GlobalRef};
use func::{FuncRef, FuncBody, FuncInstance};
use table::TableRef;
use memory::MemoryRef;
use host::Externals;
use common::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX};
use types::{GlobalDescriptor, TableDescriptor, MemoryDescriptor};

#[derive(Clone, Debug)]
pub struct ModuleRef(pub(crate) Rc<ModuleInstance>);

impl ::std::ops::Deref for ModuleRef {
	type Target = ModuleInstance;
	fn deref(&self) -> &ModuleInstance {
		&self.0
	}
}

/// An external value is the runtime representation of an entity that can be imported or exported.
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
	/// Get underlying function reference if this `ExternVal` contains
	/// a function, or `None` if it is some other kind.
	pub fn as_func(&self) -> Option<&FuncRef> {
		match *self {
			ExternVal::Func(ref func) => Some(func),
			_ => None,
		}
	}

	/// Get underlying table reference if this `ExternVal` contains
	/// a table, or `None` if it is some other kind.
	pub fn as_table(&self) -> Option<&TableRef> {
		match *self {
			ExternVal::Table(ref table) => Some(table),
			_ => None,
		}
	}

	/// Get underlying memory reference if this `ExternVal` contains
	/// a memory, or `None` if it is some other kind.
	pub fn as_memory(&self) -> Option<&MemoryRef> {
		match *self {
			ExternVal::Memory(ref memory) => Some(memory),
			_ => None,
		}
	}

	/// Get underlying global variable reference if this `ExternVal` contains
	/// a global, or `None` if it is some other kind.
	pub fn as_global(&self) -> Option<&GlobalRef> {
		match *self {
			ExternVal::Global(ref global) => Some(global),
			_ => None,
		}
	}
}

/// A module instance is the runtime representation of a [module][`LoadedModule`].
///
/// It is created by instantiating a [module][`LoadedModule`], and collects runtime representations
/// of all entities that are imported or defined by the module, namely:
///
/// - [functions][`FuncInstance`],
/// - [memories][`MemoryInstance`],
/// - [tables][`TableInstance`],
/// - [globals][`GlobalInstance`],
///
/// In order to instantiate a module you need to provide entities to satisfy
/// every module's imports (i.e. wasm modules don't have optional imports).
///
/// After module is instantiated you can start invoking it's exported functions with [`invoke_export`].
///
/// [`LoadedModule`]: struct.LoadedModule.html
/// [`FuncInstance`]: struct.FuncInstance.html
/// [`MemoryInstance`]: struct.MemoryInstance.html
/// [`TableInstance`]: struct.TableInstance.html
/// [`GlobalInstance`]: struct.GlobalInstance.html
/// [`invoke_export`]: #method.invoke_export
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
		loaded_module: &LoadedModule,
		extern_vals: &[ExternVal]
	) -> Result<ModuleRef, Error> {
		let module = loaded_module.module();
		let instance = ModuleRef(Rc::new(ModuleInstance::default()));

		for &Type::Function(ref ty) in module.type_section().map(|ts| ts.types()).unwrap_or(&[]) {
			let signature = Rc::new(Signature::from_elements(ty));
			instance.push_signature(signature);
		}

		{
			let imports = module.import_section().map(|is| is.entries()).unwrap_or(
				&[],
			);
			if imports.len() != extern_vals.len() {
				return Err(Error::Instantiation(
					"extern_vals length is not equal to import section entries".to_owned()
				));
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

		let labels = loaded_module.labels();
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
					FuncInstance::alloc_internal(Rc::downgrade(&instance.0), signature, func_body);
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
		loaded_module: &LoadedModule,
		extern_vals: &[ExternVal],
	) -> Result<ModuleRef, Error> {
		let module = loaded_module.module();

		let module_ref = ModuleInstance::alloc_module(loaded_module, extern_vals)?;

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

	/// Instantiate a [module][`LoadedModule`].
	///
	/// Note that in case of successful instantiation this function returns a reference to
	/// a module which `start` function is not called.
	/// In order to complete instantiatiation `start` function must be called. However, there are
	/// situations where you might need to do additional setup before calling `start` function.
	/// For such sitations this separation might be useful.
	///
	/// # Errors
	///
	/// Returns `Err` if the module cannot be instantiated.
	///
	/// This can happen if one of the imports can't
	/// be satisfied (e.g module isn't registered in `imports` [resolver][`ImportResolver`]) or
	/// there is a mismatch between requested import and provided (e.g. module requested memory with no
	/// maximum size limit, however, was provided memory with the maximum size limit).
	///
	/// # Examples
	///
	/// ```rust
	/// use wasmi::{load_from_buffer, ModuleInstance, ImportsBuilder, NopExternals};
	/// # fn func() -> Result<(), ::wasmi::Error> {
	/// # let module = load_from_buffer(&[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]).unwrap();
	///
	/// // ModuleInstance::new returns instance which `start` function isn't called.
	/// let not_started = ModuleInstance::new(
	///		&module,
	///		&ImportsBuilder::default()
	///	)?;
	/// // Call `start` function if any.
	/// let instance = not_started.run_start(&mut NopExternals)?;
	///
	/// # Ok(())
	/// # }
	/// ```
	///
	/// If you sure that the module doesn't have `start` function you can use [`assert_no_start`] to get
	/// instantiated module without calling `start` function.
	///
	/// ```rust
	/// use wasmi::{load_from_buffer, ModuleInstance, ImportsBuilder, NopExternals};
	/// # fn func() -> Result<(), ::wasmi::Error> {
	/// # let module = load_from_buffer(&[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]).unwrap();
	///
	/// // This will panic if the module actually contain `start` function.
	/// let not_started = ModuleInstance::new(
	///		&module,
	///		&ImportsBuilder::default()
	///	)?.assert_no_start();
	///
	/// # Ok(())
	/// # }
	/// ```
	///
	/// [`LoadedModule`]: struct.LoadedModule.html
	/// [`ImportResolver`]: trait.ImportResolver.html
	/// [`assert_no_start`]: struct.NotStartedModuleRef.html#method.assert_no_start
	pub fn new<'m, I: ImportResolver>(
		loaded_module: &'m LoadedModule,
		imports: &I,
	) -> Result<NotStartedModuleRef<'m>, Error> {
		let module = loaded_module.module();

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
					let signature = Signature::from_elements(func_type);
					let func = imports.resolve_func(module_name, field_name, &signature)?;
					ExternVal::Func(func)
				}
				External::Table(ref table_type) => {
					let table_descriptor = TableDescriptor::from_elements(table_type);
					let table = imports.resolve_table(module_name, field_name, &table_descriptor)?;
					ExternVal::Table(table)
				}
				External::Memory(ref memory_type) => {
					let memory_descriptor = MemoryDescriptor::from_elements(memory_type);
					let memory = imports.resolve_memory(module_name, field_name, &memory_descriptor)?;
					ExternVal::Memory(memory)
				}
				External::Global(ref global_type) => {
					let global_descriptor = GlobalDescriptor::from_elements(global_type);
					let global = imports.resolve_global(module_name, field_name, &global_descriptor)?;
					ExternVal::Global(global)
				}
			};
			extern_vals.push(extern_val);
		}

		let instance = Self::instantiate_with_externvals(loaded_module, &extern_vals)?;
		Ok(NotStartedModuleRef {
			loaded_module,
			instance,
		})
	}

	/// Invoke exported function by a name.
	///
	/// This function finds exported function by a name, and calls it with provided arguments and
	/// external state.
	///
	/// # Errors
	///
	/// Returns `Err` if:
	///
	/// - there are no export with a given name or this export is not a function,
	/// - given arguments doesn't match to function signature,
	/// - trap occured at the execution time,
	///
	/// # Examples
	///
	/// Invoke a function that takes two numbers and returns sum of them.
	///
	/// ```rust
	/// # extern crate wasmi;
	/// # extern crate wabt;
	/// # use wasmi::{ModuleInstance, ImportsBuilder, NopExternals, RuntimeValue};
	/// # fn main() {
	///	# let wasm_binary: Vec<u8> = wabt::wat2wasm(
	///	# 	r#"
	///	# 	(module
	///	# 		(func (export "add") (param i32 i32) (result i32)
	/// # 			get_local 0
	/// # 			get_local 1
	///	# 			i32.add
	///	# 		)
	///	# 	)
	///	# 	"#,
	///	# ).expect("failed to parse wat");
	/// # let module = wasmi::load_from_buffer(&wasm_binary).expect("failed to load wasm");
	/// # let instance = ModuleInstance::new(
	///	# &module,
	///	# &ImportsBuilder::default()
	///	# ).expect("failed to instantiate wasm module").assert_no_start();
	/// assert_eq!(
	/// 	instance.invoke_export(
	/// 		"add",
	/// 		&[RuntimeValue::I32(5), RuntimeValue::I32(3)],
	///			&mut NopExternals,
	///		).expect("failed to execute export"),
	///		Some(RuntimeValue::I32(8)),
	///	);
	/// # }
	/// ```
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

		FuncInstance::invoke(&func_instance, args, externals)
	}

	/// Find export by a name.
	///
	/// Returns `None` if there is no export with such name.
	pub fn export_by_name(&self, name: &str) -> Option<ExternVal> {
		self.exports.borrow().get(name).cloned()
	}
}

pub struct NotStartedModuleRef<'a> {
	loaded_module: &'a LoadedModule,
	instance: ModuleRef,
}

impl<'a> NotStartedModuleRef<'a> {
	pub fn not_started_instance(&self) -> &ModuleRef {
		&self.instance
	}

	pub fn run_start<E: Externals>(self, state: &mut E) -> Result<ModuleRef, Error> {
		if let Some(start_fn_idx) = self.loaded_module.module().start_section() {
			let start_func = self.instance.func_by_index(start_fn_idx).expect(
				"Due to validation start function should exists",
			);
			FuncInstance::invoke(&start_func, &[], state)?;
		}
		Ok(self.instance)
	}

	pub fn assert_no_start(self) -> ModuleRef {
		assert!(self.loaded_module.module().start_section().is_none());
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
