mod error;
mod pre;

pub use self::{error::InstantiationError, pre::InstancePre};
use super::{export, InitExpr, Module, ModuleImportType};
use crate::{
    module::{init_expr::InitExprOperand, DEFAULT_MEMORY_INDEX},
    AsContext,
    AsContextMut,
    Error,
    Extern,
    FuncEntity,
    FuncType,
    Global,
    GlobalType,
    Instance,
    InstanceEntity,
    InstanceEntityBuilder,
    Memory,
    MemoryType,
    Mutability,
    Table,
    TableType,
};
use wasmi_core::{Value, ValueType, F32, F64};

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
        Ok(InstancePre::new(handle, self, builder))
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
        _context: &mut impl AsContextMut,
        builder: &mut InstanceEntityBuilder,
    ) {
        for func_type in self.func_types() {
            builder.push_func_type(*func_type);
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
        let mut imports = self.imports();
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
            match (import.item_type(), external) {
                (ModuleImportType::Func(expected_signature), Extern::Func(func)) => {
                    let actual_signature = func.signature(context.as_context());
                    // Note: We can compare function signatures without resolving them because
                    //       we deduplicate them before registering. Therefore two equal instances of
                    //       [`SignatureEntity`] will be associated to the same [`Signature`].
                    if &actual_signature != expected_signature {
                        // Note: In case of error we could resolve the signatures for better error readability.
                        return Err(InstantiationError::SignatureMismatch {
                            actual: actual_signature,
                            expected: *expected_signature,
                        });
                    }
                    builder.push_func(func);
                }
                (ModuleImportType::Table(required), Extern::Table(table)) => {
                    let imported = table.table_type(context.as_context());
                    imported.satisfies(required)?;
                    builder.push_table(table);
                }
                (ModuleImportType::Memory(required), Extern::Memory(memory)) => {
                    let imported = memory.memory_type(context.as_context());
                    imported.satisfies(required)?;
                    builder.push_memory(memory);
                }
                (ModuleImportType::Global(expected), Extern::Global(global)) => {
                    let expected = *expected;
                    let actual = global.global_type(&context);
                    if expected != actual {
                        return Err(InstantiationError::GlobalTypeMismatch { expected, actual });
                    }
                    builder.push_global(global);
                }
                (expected_import, actual_extern_val) => {
                    return Err(InstantiationError::ImportsExternalsMismatch {
                        expected: expected_import.clone(),
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
        for (func_type, func_body) in self.internal_funcs() {
            let func = context
                .as_context_mut()
                .store
                .alloc_func(FuncEntity::new_wasm(func_type, func_body, handle));
            builder.push_func(func);
        }
    }

    /// Extracts the Wasm tables from the module and stores them into the [`Store`].
    ///
    /// This also stores [`Table`] references into the [`Instance`] under construction.
    ///
    /// [`Store`]: struct.Store.html
    fn extract_tables(&self, context: &mut impl AsContextMut, builder: &mut InstanceEntityBuilder) {
        for table_type in self.tables.iter().copied() {
            builder.push_table(Table::new(context.as_context_mut(), table_type));
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
        for memory_type in self.memories.iter().copied() {
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
        for (global_type, global_init) in self.internal_globals() {
            let init_value = Self::eval_init_expr(context.as_context_mut(), builder, global_init);
            let mutability = global_type.mutability();
            let global = Global::new(context.as_context_mut(), init_value, mutability);
            builder.push_global(global);
        }
    }

    /// Evaluates the given initializer expression using the partially constructed [`Instance`].
    fn eval_init_expr(
        context: impl AsContext,
        builder: &InstanceEntityBuilder,
        init_expr: &InitExpr,
    ) -> Value {
        let operands = init_expr.operators();
        debug_assert_eq!(
            operands.len(),
            1,
            "in Wasm MVP code length of initializer expressions must be 1 but found {} operands",
            operands.len(),
        );
        match operands[0] {
            InitExprOperand::Const(value) => value,
            InitExprOperand::GlobalGet(global_index) => {
                let global = builder
                    .get_global(global_index.into_u32())
                    .unwrap_or_else(|| {
                        panic!(
                            "encountered missing global at index {:?} for initializer expression evaluation",
                            global_index
                        )
                    });
                global.get(context)
            }
        }
    }

    /// Extracts the Wasm exports from the module and registers them into the [`Instance`].
    fn extract_exports(&self, builder: &mut InstanceEntityBuilder) {
        for export in &self.exports[..] {
            let field = export.field();
            let external = match export.external() {
                export::External::Func(func_index) => {
                    let func_index = func_index.into_u32();
                    let func = builder.get_func(func_index).unwrap_or_else(|| {
                        panic!(
                            "encountered missing function at index {:?} upon element initialization",
                            func_index,
                        )
                    });
                    Extern::Func(func)
                }
                export::External::Table(table_index) => {
                    let table_index = table_index.into_u32();
                    let table = builder.get_table(table_index).unwrap_or_else(|| {
                        panic!(
                            "encountered missing table at index {:?} upon element initialization",
                            table_index,
                        )
                    });
                    Extern::Table(table)
                }
                export::External::Memory(memory_index) => {
                    let memory_index = memory_index.into_u32();
                    let memory = builder.get_memory(memory_index).unwrap_or_else(|| {
                        panic!(
                            "encountered missing memory at index {:?} upon element initialization",
                            memory_index,
                        )
                    });
                    Extern::Memory(memory)
                }
                export::External::Global(global_index) => {
                    let global_index = global_index.into_u32();
                    let global = builder.get_global(global_index).unwrap_or_else(|| {
                        panic!(
                            "encountered missing global at index {:?} upon element initialization",
                            global_index,
                        )
                    });
                    Extern::Global(global)
                }
            };
            builder.push_export(field, external);
        }
    }

    /// The index of the default Wasm table.
    const DEFAULT_TABLE_INDEX: u32 = 0;

    /// Initializes the [`Instance`] tables with the Wasm element segments of the [`Module`].
    fn initialize_table_elements(
        &self,
        context: &mut impl AsContextMut,
        builder: &mut InstanceEntityBuilder,
    ) -> Result<(), Error> {
        for element_segment in &self.element_segments[..] {
            let offset_expr = element_segment.offset();
            let offset = Self::eval_init_expr(context.as_context_mut(), builder, offset_expr)
                .try_into::<u32>()
                .unwrap_or_else(|| {
                    panic!(
                    "expected offset value of type `i32` due to Wasm validation but found: {:?}",
                    offset_expr,
                )
                }) as usize;
            let table = builder
                .get_table(Self::DEFAULT_TABLE_INDEX)
                .unwrap_or_else(|| {
                    panic!(
                        "expected default table at index {} but found none",
                        Self::DEFAULT_TABLE_INDEX
                    )
                });
            // Note: This checks not only that the elements in the element segments properly
            //       fit into the table at the given offset but also that the element segment
            //       consists of at least 1 element member.
            let len_table = table.len(&context);
            let len_items = element_segment.items().len();
            if offset + len_items > len_table {
                return Err(InstantiationError::ElementSegmentDoesNotFit {
                    table,
                    offset,
                    amount: len_items,
                })
                .map_err(Into::into);
            }
            // Finally do the actual initialization of the table elements.
            for (i, func_index) in element_segment.items().iter().enumerate() {
                let func_index = func_index.into_u32();
                let func = builder.get_func(func_index).unwrap_or_else(|| {
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
        for data_segment in &self.data_segments[..] {
            let offset_expr = data_segment.offset();
            let offset = Self::eval_init_expr(context.as_context_mut(), builder, offset_expr)
                .try_into::<u32>()
                .unwrap_or_else(|| {
                    panic!(
                    "expected offset value of type `i32` due to Wasm validation but found: {:?}",
                    offset_expr,
                )
                }) as usize;
            let memory = builder
                .get_memory(Self::DEFAULT_TABLE_INDEX)
                .unwrap_or_else(|| {
                    panic!(
                        "expected default memory at index {} but found none",
                        DEFAULT_MEMORY_INDEX
                    )
                });
            memory.write(context.as_context_mut(), offset, data_segment.data())?;
        }
        Ok(())
    }
}
