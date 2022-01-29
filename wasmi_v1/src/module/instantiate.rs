//! We use a dedicated submodule for functions belonging to the instantiation process.
//!
//! The reason for this is that the instantiation process is complex and therefore
//! it helps to split it up into several utility functions. Putting all those functions
//! into a submodule and marking them as private won't allow access to them from other
//! parts of the [`Module`] API internally.
//!
//! In summary: This improves encapsulation of implementation details.

use super::{
    super::{
        engine::DedupFuncType,
        errors::{MemoryError, TableError},
        AsContext,
        AsContextMut,
        Error,
        Extern,
        FuncEntity,
        FuncType,
        Global,
        Instance,
        InstanceEntity,
        InstanceEntityBuilder,
        Memory,
        MemoryType,
        Mutability,
        Table,
        TableType,
    },
    Module,
};
use crate::{
    core::{Value, ValueType, F32, F64},
    GlobalType,
};
use core::{fmt, fmt::Display};
use parity_wasm::elements as pwasm;
use validation::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX};

/// An error that may occur upon instantiation of a Wasm module.
#[derive(Debug)]
pub enum InstantiationError {
    /// Caused when the number of required imports does not match
    /// the number of given externals upon module instantiation.
    ImportsExternalsLenMismatch,
    /// Caused when a given external value does not match the
    /// type of the required import for module instantiation.
    ImportsExternalsMismatch {
        /// The expected external value for the module import.
        expected: pwasm::External,
        /// The actually found external value for the module import.
        actual: Extern,
    },
    /// Caused when a function has a mismatching signature.
    SignatureMismatch {
        /// The expected function signature for the function import.
        expected: DedupFuncType,
        /// The actual function signature for the function import.
        actual: DedupFuncType,
    },
    /// Occurs when an imported table does not satisfy the required table type.
    Table(TableError),
    /// Occurs when an imported memory does not satisfy the required memory type.
    Memory(MemoryError),
    /// Caused when a global variable has a mismatching global variable type and mutability.
    GlobalTypeMismatch {
        /// The expected global type for the global variable import.
        expected: GlobalType,
        /// The actual global type found for the global variable import.
        actual: GlobalType,
    },
    /// Caused when an element segment does not fit into the specified table instance.
    ElementSegmentDoesNotFit {
        /// The table of the element segment.
        table: Table,
        /// The offset to store the `amount` of elements into the table.
        offset: usize,
        /// The amount of elements with which the table is initialized at the `offset`.
        amount: usize,
    },
    /// Caused when the `start` function was unexpectedly found in the instantiated module.
    FoundStartFn {
        /// The index of the found `start` function.
        index: u32,
    },
}

#[cfg(feature = "std")]
impl std::error::Error for InstantiationError {}

impl Display for InstantiationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ImportsExternalsLenMismatch => write!(
                f,
                "encountered mismatch between number of given externals and module imports",
            ),
            Self::ImportsExternalsMismatch { expected, actual } => write!(
                f,
                "expected {:?} external for import but found {:?}",
                expected, actual
            ),
            Self::SignatureMismatch { expected, actual } => {
                write!(
                    f,
                    "expected {:?} function signature but found {:?}",
                    expected, actual
                )
            }
            Self::GlobalTypeMismatch { expected, actual } => write!(
                f,
                "expected {:?} global type but found {:?} value type",
                expected, actual,
            ),
            Self::ElementSegmentDoesNotFit {
                table,
                offset,
                amount,
            } => write!(
                f,
                "table {:?} does not fit {} elements starting from offset {}",
                table, offset, amount,
            ),
            Self::FoundStartFn { index } => {
                write!(f, "found an unexpected start function with index {}", index)
            }
            Self::Table(error) => Display::fmt(error, f),
            Self::Memory(error) => Display::fmt(error, f),
        }
    }
}

impl From<TableError> for InstantiationError {
    fn from(error: TableError) -> Self {
        Self::Table(error)
    }
}

impl From<MemoryError> for InstantiationError {
    fn from(error: MemoryError) -> Self {
        Self::Memory(error)
    }
}

/// A partially instantiated [`Instance`] where the `start` function has not yet been executed.
///
/// # Note
///
/// Some users require Wasm modules to not have a `start` function that is required for
/// conformant module instantiation. This API provides control over the precise instantiation
/// process with regard to this need.
#[derive(Debug)]
pub struct InstancePre<'a> {
    handle: Instance,
    module: &'a Module,
    builder: InstanceEntityBuilder,
}

impl<'a> InstancePre<'a> {
    /// Returns the index of the `start` function if any.
    ///
    /// Returns `None` if the [`Module`] does not have a `start` function.
    fn start_fn(&self) -> Option<u32> {
        self.module.module.start_section()
    }

    /// Runs the `start` function of the [`Instance`] and returns its handle.
    ///
    /// # Note
    ///
    /// This finishes the instantiation procedure.
    ///
    /// # Errors
    ///
    /// If executing the `start` function traps.
    ///
    /// # Panics
    ///
    /// If the `start` function is invalid albeit successful validation.
    pub fn start(self, mut context: impl AsContextMut) -> Result<Instance, Error> {
        let opt_start_index = self.start_fn();
        context
            .as_context_mut()
            .store
            .initialize_instance(self.handle, self.builder.finish());
        if let Some(start_index) = opt_start_index {
            let start_func = self
                .handle
                .get_func(&mut context, start_index)
                .unwrap_or_else(|| {
                    panic!(
                        "encountered invalid start function after validation: {}",
                        start_index
                    )
                });
            start_func.call(context.as_context_mut(), &[], &mut [])?
        }
        Ok(self.handle)
    }

    /// Finishes instantiation ensuring that no `start` function exists.
    ///
    /// # Errors
    ///
    /// If a `start` function exists that needs to be called for conformant module instantiation.
    pub fn ensure_no_start(
        self,
        mut context: impl AsContextMut,
    ) -> Result<Instance, InstantiationError> {
        if let Some(index) = self.start_fn() {
            return Err(InstantiationError::FoundStartFn { index });
        }
        context
            .as_context_mut()
            .store
            .initialize_instance(self.handle, self.builder.finish());
        Ok(self.handle)
    }
}

impl Module {
    /// Instantiates a new [`Instance`] from the given compiled [`Module`].
    ///
    /// Uses the given `context` to store the instance data to.
    /// The given `externals` are joned with the imports in the same order in which they occure.
    ///
    /// # Note
    ///
    /// This is a very low-level API. For a more high-level API users should use the
    /// corresponding instantiation methods provided by the [`Linker`].
    ///
    /// # Errors
    ///
    /// If the given `externals` do not satisfy the required imports, e.g. if an externally
    /// provided [`Func`] has a different function signature than required by the module import.
    ///
    /// [`Linker`]: struct.Linker.html
    /// [`Func`]: [`crate::v1::Func`]
    pub(crate) fn instantiate<I>(
        &self,
        mut context: impl AsContextMut,
        externals: I,
    ) -> Result<InstancePre, Error>
    where
        I: IntoIterator<Item = Extern>,
    {
        let handle = context.as_context_mut().store.alloc_instance();
        let mut builder = InstanceEntity::build();

        self.extract_func_types(&mut context, &mut builder);
        self.extract_imports(&mut context, &mut builder, externals)?;
        self.extract_functions(&mut context, &mut builder, handle);
        self.extract_tables(&mut context, &mut builder);
        self.extract_memories(&mut context, &mut builder);
        self.extract_globals(&mut context, &mut builder);
        self.extract_exports(&mut builder);

        self.initialize_table_elements(&mut context, &mut builder)?;
        self.initialize_memory_data(&mut context, &mut builder)?;

        // At this point the module instantiation is nearly done.
        // The only thing that is missing is to run the `start` function.

        Ok(InstancePre {
            handle,
            module: self,
            builder,
        })
    }

    /// Extracts the Wasm function signatures from the
    /// module and stores them into the [`Store`].
    ///
    /// This also stores deduplicated [`FuncType`] references into the
    /// [`Instance`] under construction.
    ///
    /// [`Store`]: struct.Store.html
    fn extract_func_types(
        &self,
        context: &mut impl AsContextMut,
        builder: &mut InstanceEntityBuilder,
    ) {
        let types = self
            .module
            .type_section()
            .map(pwasm::TypeSection::types)
            .unwrap_or(&[]);
        for pwasm::Type::Function(func_type) in types {
            let inputs = func_type
                .params()
                .iter()
                .copied()
                .map(ValueType::from_elements);
            let outputs = func_type
                .results()
                .iter()
                .copied()
                .map(ValueType::from_elements);
            let signature = context
                .as_context_mut()
                .store
                .alloc_func_type(FuncType::new(inputs, outputs));
            builder.push_func_type(signature);
        }
    }

    /// Extract the Wasm imports from the module and zips them with the given external values.
    ///
    /// This also stores imported references into the [`Instance`] under construction.
    ///
    /// # Errors
    ///
    /// - If too few or too many external values are given for the required module imports.
    /// - If the zipped import and given external have mismatching types, e.g. on index `i`
    ///   the module requires a function import but on index `i` the externals provide a global
    ///   variable external value.
    /// - If the externally provided [`Table`], [`Memory`], [`Func`] or [`Global`] has a type
    ///   mismatch with the expected module import type.
    ///
    /// [`Func`]: [`crate::v1::Func`]
    fn extract_imports<I>(
        &self,
        context: &mut impl AsContextMut,
        builder: &mut InstanceEntityBuilder,
        externals: I,
    ) -> Result<(), InstantiationError>
    where
        I: IntoIterator<Item = Extern>,
    {
        let mut imports = self
            .module
            .import_section()
            .map(pwasm::ImportSection::entries)
            .unwrap_or(&[])
            .iter();
        let mut externals = externals.into_iter();
        loop {
            // Iterate on module imports and the given external values in lock-step fashion.
            //
            // Note: We cannot use [`zip`](`core::iter::zip`) here since we require that both
            //       iterators yield the same amount of elements.
            let (import, external) = match (imports.next(), externals.next()) {
                (Some(import), Some(external)) => (import, external),
                (None, None) => break,
                (Some(_), None) | (None, Some(_)) => {
                    return Err(InstantiationError::ImportsExternalsLenMismatch)
                }
            };
            match (import.external(), external) {
                (pwasm::External::Function(signature_index), Extern::Func(func)) => {
                    let expected_signature =
                        builder.get_signature(*signature_index).unwrap_or_else(|| {
                            panic!(
                                "expected function signature at index {} due to validation",
                                signature_index
                            )
                        });
                    let actual_signature = func.signature(context.as_context());
                    // Note: We can compare function signatures without resolving them because
                    //       we deduplicate them before registering. Therefore two equal instances of
                    //       [`SignatureEntity`] will be associated to the same [`Signature`].
                    if expected_signature != actual_signature {
                        // Note: In case of error we could resolve the signatures for better error readability.
                        return Err(InstantiationError::SignatureMismatch {
                            actual: actual_signature,
                            expected: expected_signature,
                        });
                    }
                    builder.push_func(func);
                }
                (pwasm::External::Table(table_type), Extern::Table(table)) => {
                    let required = TableType::from_elements(table_type);
                    let imported = table.table_type(context.as_context());
                    imported.satisfies(&required)?;
                    builder.push_table(table);
                }
                (pwasm::External::Memory(memory_type), Extern::Memory(memory)) => {
                    let required = MemoryType::from_elements(memory_type);
                    let imported = memory.memory_type(context.as_context());
                    imported.satisfies(&required)?;
                    builder.push_memory(memory);
                }
                (pwasm::External::Global(global_type), Extern::Global(global)) => {
                    let expected = GlobalType::from_elements(*global_type);
                    let actual = global.global_type(&context);
                    if expected != actual {
                        return Err(InstantiationError::GlobalTypeMismatch { expected, actual });
                    }
                    builder.push_global(global);
                }
                (expected_import, actual_extern_val) => {
                    return Err(InstantiationError::ImportsExternalsMismatch {
                        expected: *expected_import,
                        actual: actual_extern_val,
                    });
                }
            }
        }
        Ok(())
    }

    /// Extracts the Wasm functions from the module and stores them into the [`Store`].
    ///
    /// This also stores [`Func`] references into the [`Instance`] under construction.
    ///
    /// [`Store`]: struct.Store.html
    /// [`Func`]: [`crate::v1::Func`]
    fn extract_functions(
        &self,
        context: &mut impl AsContextMut,
        builder: &mut InstanceEntityBuilder,
        handle: Instance,
    ) {
        let func_bodies = &self.func_bodies[..];
        let funcs = self
            .module
            .function_section()
            .map(pwasm::FunctionSection::entries)
            .unwrap_or(&[]);
        let wasm_bodies = self
            .module
            .code_section()
            .map(pwasm::CodeSection::bodies)
            .unwrap_or(&[]);
        assert!(
            funcs.len() == wasm_bodies.len(),
            "due to Wasm validation function and function body counts must match \
            but found {} functions and {} bodies",
            funcs.len(),
            wasm_bodies.len(),
        );
        assert!(
            wasm_bodies.len() == func_bodies.len(),
            "due to Wasm validation counts for Wasm function bodies and `wasmi` function bodies \
            must match but found {} Wasm function bodies and {} `wamsi` function bodies",
            wasm_bodies.len(),
            func_bodies.len(),
        );
        for (func_type, func_body) in funcs.iter().zip(func_bodies.iter()) {
            let signature_index = func_type.type_ref();
            let signature = builder.get_signature(signature_index).unwrap_or_else(|| {
                panic!(
                    "encountered missing function signature in instance for index {}",
                    signature_index
                )
            });
            let func = context
                .as_context_mut()
                .store
                .alloc_func(FuncEntity::new_wasm(signature, *func_body, handle));
            builder.push_func(func);
        }
    }

    /// Extracts the Wasm tables from the module and stores them into the [`Store`].
    ///
    /// This also stores [`Table`] references into the [`Instance`] under construction.
    ///
    /// [`Store`]: struct.Store.html
    fn extract_tables(&self, context: &mut impl AsContextMut, builder: &mut InstanceEntityBuilder) {
        let table_types = self
            .module
            .table_section()
            .map(pwasm::TableSection::entries)
            .unwrap_or(&[]);
        for table_type in table_types {
            let table_type = TableType::from_elements(table_type);
            let table = Table::new(context.as_context_mut(), table_type);
            builder.push_table(table);
        }
    }

    /// Extracts the Wasm linear memories from the module and stores them into the [`Store`].
    ///
    /// This also stores [`Memory`] references into the [`Instance`] under construction.
    ///
    /// [`Store`]: struct.Store.html
    fn extract_memories(
        &self,
        context: &mut impl AsContextMut,
        builder: &mut InstanceEntityBuilder,
    ) {
        let memory_types = self
            .module
            .memory_section()
            .map(pwasm::MemorySection::entries)
            .unwrap_or(&[]);
        for memory_type in memory_types {
            let memory_type = MemoryType::from_elements(memory_type);
            let memory =
                Memory::new(context.as_context_mut(), memory_type).unwrap_or_else(|error| {
                    panic!(
                        "encountered unexpected invalid memory type {:?} after Wasm validation: {}",
                        memory_type, error,
                    )
                });
            builder.push_memory(memory);
        }
    }

    /// Extracts the Wasm global variables from the module and stores them into the [`Store`].
    ///
    /// This also stores [`Global`] references into the [`Instance`] under construction.
    ///
    /// [`Store`]: struct.Store.html
    fn extract_globals(
        &self,
        context: &mut impl AsContextMut,
        builder: &mut InstanceEntityBuilder,
    ) {
        let global_entries = self
            .module
            .global_section()
            .map(pwasm::GlobalSection::entries)
            .unwrap_or(&[]);
        for global_entry in global_entries {
            let init_value =
                Self::eval_init_expr(context.as_context(), builder, global_entry.init_expr());
            let global_type = ValueType::from_elements(global_entry.global_type().content_type());
            debug_assert_eq!(
                init_value.value_type(),
                global_type,
                "encountered mismatch between global variable init value {:?} and expected global value type {:?}",
                init_value,
                global_type,
            );
            let mutability = match global_entry.global_type().is_mutable() {
                true => Mutability::Mutable,
                false => Mutability::Const,
            };
            let global = Global::new(context.as_context_mut(), init_value, mutability);
            builder.push_global(global);
        }
    }

    /// Extracts the Wasm exports from the module and registers them into the [`Instance`].
    fn extract_exports(&self, builder: &mut InstanceEntityBuilder) {
        let exports = self
            .module
            .export_section()
            .map(pwasm::ExportSection::entries)
            .unwrap_or(&[]);
        for export in exports {
            let field = export.field();
            let extern_val = match export.internal() {
                pwasm::Internal::Function(index) => {
                    let func = builder.get_func(*index).unwrap_or_else(|| {
                        panic!("encountered missing exported function at index {}", index)
                    });
                    Extern::Func(func)
                }
                pwasm::Internal::Global(index) => {
                    let global = builder.get_global(*index).unwrap_or_else(|| {
                        panic!(
                            "encountered missing exported global variable at index {}",
                            index
                        )
                    });
                    Extern::Global(global)
                }
                pwasm::Internal::Memory(index) => {
                    let memory = builder.get_memory(*index).unwrap_or_else(|| {
                        panic!(
                            "encountered missing exported linear memory at index {}",
                            index
                        )
                    });
                    Extern::Memory(memory)
                }
                pwasm::Internal::Table(index) => {
                    let table = builder.get_table(*index).unwrap_or_else(|| {
                        panic!("encountered missing exported table at index {}", index)
                    });
                    Extern::Table(table)
                }
            };
            builder.push_export(field, extern_val);
        }
    }

    /// Evaluates the given initializer expression using the partially constructed [`Instance`].
    fn eval_init_expr(
        context: impl AsContext,
        builder: &InstanceEntityBuilder,
        init_expr: &pwasm::InitExpr,
    ) -> Value {
        let operands = init_expr.code();
        debug_assert_eq!(
            operands.len(),
            2,
            "in Wasm MVP code length of initializer expressions must be 2 but found {} operands",
            operands.len(),
        );
        debug_assert!(matches!(operands[1], pwasm::Instruction::End));
        match &operands[0] {
            pwasm::Instruction::I32Const(value) => Value::from(*value),
            pwasm::Instruction::I64Const(value) => Value::from(*value),
            pwasm::Instruction::F32Const(value) => Value::from(F32::from_bits(*value)),
            pwasm::Instruction::F64Const(value) => Value::from(F64::from_bits(*value)),
            pwasm::Instruction::GetGlobal(global_index) => {
                let global = builder
                    .get_global(*global_index)
                    .unwrap_or_else(|| {
                        panic!(
                            "encountered missing global at index {} for initializer expression evaluation",
                            global_index
                        )
                    });
                global.get(context)
            }
            unexpected => panic!(
                "encountered unexpected operand for initializer expression: {:?}",
                unexpected
            ),
        }
    }

    /// Initializes the [`Instance`] tables with the Wasm element segments of the [`Module`].
    fn initialize_table_elements(
        &self,
        context: &mut impl AsContextMut,
        builder: &mut InstanceEntityBuilder,
    ) -> Result<(), Error> {
        let element_segments = self
            .module
            .elements_section()
            .map(pwasm::ElementSection::entries)
            .unwrap_or(&[]);
        for element_segment in element_segments {
            let offset_expr = element_segment.offset().as_ref().unwrap_or_else(|| {
                panic!(
                    "encountered unsupported passive element segment: {:?}",
                    element_segment
                )
            });
            let offset = Self::eval_init_expr(context.as_context(), builder, offset_expr)
                .try_into::<u32>()
                .unwrap_or_else(|| {
                    panic!(
                    "expected offset value of type `i32` due to Wasm validation but found: {:?}",
                    offset_expr,
                )
                }) as usize;
            let table = builder.get_table(DEFAULT_TABLE_INDEX).unwrap_or_else(|| {
                panic!(
                    "expected default table at index {} but found none",
                    DEFAULT_TABLE_INDEX
                )
            });
            // Note: This checks not only that the elements in the element segments properly
            //       fit into the table at the given offset but also that the element segment
            //       consists of at least 1 element member.
            if offset + element_segment.members().len() > table.len(context.as_context()) {
                return Err(InstantiationError::ElementSegmentDoesNotFit {
                    table,
                    offset: offset as usize,
                    amount: table.len(context.as_context()),
                })
                .map_err(Into::into);
            }
            // Finally do the actual initialization of the table elements.
            for (i, func_index) in element_segment.members().iter().enumerate() {
                let func = builder.get_func(*func_index).unwrap_or_else(|| {
                    panic!(
                        "encountered missing function at index {} upon element initialization",
                        func_index
                    )
                });
                table.set(context.as_context_mut(), offset + i, Some(func))?;
            }
        }
        Ok(())
    }

    /// Initializes the [`Instance`] linear memories with the Wasm data segments of the [`Module`].
    fn initialize_memory_data(
        &self,
        context: &mut impl AsContextMut,
        builder: &mut InstanceEntityBuilder,
    ) -> Result<(), Error> {
        let data_segments = self
            .module
            .data_section()
            .map(pwasm::DataSection::entries)
            .unwrap_or(&[]);
        for data_segment in data_segments {
            let offset_expr = data_segment.offset().as_ref().unwrap_or_else(|| {
                panic!(
                    "encountered unsupported passive data segment: {:?}",
                    data_segment
                )
            });
            let offset = Self::eval_init_expr(context.as_context(), builder, offset_expr)
                .try_into::<u32>()
                .unwrap_or_else(|| {
                    panic!(
                    "expected offset value of type `i32` due to Wasm validation but found: {:?}",
                    offset_expr,
                )
                }) as usize;
            let memory = builder.get_memory(DEFAULT_MEMORY_INDEX).unwrap_or_else(|| {
                panic!(
                    "expected default linear memory at index {} but found none",
                    DEFAULT_MEMORY_INDEX
                )
            });
            memory.write(context.as_context_mut(), offset, data_segment.value())?;
        }
        Ok(())
    }
}
