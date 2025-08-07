mod error;
mod pre;

#[cfg(test)]
mod tests;

pub use self::error::InstantiationError;
#[expect(deprecated)]
pub use self::pre::InstancePre;
use super::{element::ElementSegmentKind, export, ConstExpr, InitDataSegment, Module};
use crate::{
    core::{MemoryError, UntypedVal},
    error::ErrorKind,
    func::WasmFuncEntity,
    memory::DataSegment,
    value::WithType,
    AsContext,
    AsContextMut,
    ElementSegment,
    Error,
    Extern,
    ExternType,
    Func,
    Global,
    Instance,
    InstanceEntity,
    InstanceEntityBuilder,
    Memory,
    Ref,
    Table,
    Val,
};

impl Module {
    /// Instantiates a new [`Instance`] from the given compiled [`Module`].
    ///
    /// Uses the given `context` to store the instance data to.
    /// The given `externals` are joined with the imports in the same order in which they occurred.
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
    /// [`Func`]: [`crate::Func`]
    #[expect(deprecated)]
    pub(crate) fn instantiate<I>(
        &self,
        mut context: impl AsContextMut,
        externals: I,
    ) -> Result<InstancePre, Error>
    where
        I: IntoIterator<Item = Extern, IntoIter: ExactSizeIterator>,
    {
        let mut context = context.as_context_mut().store;
        if !context.can_create_more_instances(1) {
            return Err(Error::from(InstantiationError::TooManyInstances));
        }
        let handle = context.as_context_mut().store.inner.alloc_instance();
        let mut builder = InstanceEntity::build(self);

        self.extract_imports(&context, &mut builder, externals)?;
        self.extract_functions(&mut context, &mut builder, handle);
        self.extract_tables(&mut context, &mut builder)?;
        self.extract_memories(&mut context, &mut builder)?;
        self.extract_globals(&mut context, &mut builder);
        self.extract_exports(&mut builder);
        self.extract_start_fn(&mut builder);

        self.initialize_table_elements(&mut context, &mut builder)?;
        self.initialize_memory_data(&mut context, &mut builder)?;

        // At this point the module instantiation is nearly done.
        // The only thing that is missing is to run the `start` function.
        Ok(InstancePre::new(handle, builder))
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
    /// [`Func`]: [`crate::Func`]
    fn extract_imports<I>(
        &self,
        store: impl AsContext,
        builder: &mut InstanceEntityBuilder,
        externals: I,
    ) -> Result<(), InstantiationError>
    where
        I: IntoIterator<Item = Extern, IntoIter: ExactSizeIterator>,
    {
        let imports = self.imports();
        let externals = externals.into_iter();
        if imports.len() != externals.len() {
            return Err(InstantiationError::InvalidNumberOfImports {
                required: imports.len(),
                given: externals.len(),
            });
        }
        for (import, external) in imports.zip(externals) {
            match (import.ty(), external) {
                (ExternType::Func(expected_signature), Extern::Func(func)) => {
                    let actual_signature = func.ty(&store);
                    if &actual_signature != expected_signature {
                        return Err(InstantiationError::FuncTypeMismatch {
                            actual: actual_signature,
                            expected: expected_signature.clone(),
                        });
                    }
                    builder.push_func(func);
                }
                (ExternType::Table(required), Extern::Table(table)) => {
                    let imported = table.dynamic_ty(&store);
                    if !imported.is_subtype_of(required) {
                        return Err(InstantiationError::TableTypeMismatch {
                            expected: *required,
                            actual: imported,
                        });
                    }
                    builder.push_table(table);
                }
                (ExternType::Memory(required), Extern::Memory(memory)) => {
                    let imported = memory.dynamic_ty(&store);
                    if !imported.is_subtype_of(required) {
                        return Err(InstantiationError::MemoryTypeMismatch {
                            expected: *required,
                            actual: imported,
                        });
                    }
                    builder.push_memory(memory);
                }
                (ExternType::Global(required), Extern::Global(global)) => {
                    let imported = global.ty(&store);
                    let required = *required;
                    if imported != required {
                        return Err(InstantiationError::GlobalTypeMismatch {
                            expected: required,
                            actual: imported,
                        });
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
    /// [`Func`]: [`crate::Func`]
    fn extract_functions(
        &self,
        mut context: impl AsContextMut,
        builder: &mut InstanceEntityBuilder,
        handle: Instance,
    ) {
        for (func_type, func_body) in self.internal_funcs() {
            let wasm_func = WasmFuncEntity::new(func_type, func_body, handle);
            let func = context
                .as_context_mut()
                .store
                .inner
                .alloc_func(wasm_func.into());
            builder.push_func(func);
        }
    }

    /// Extracts the Wasm tables from the module and stores them into the [`Store`].
    ///
    /// This also stores [`Table`] references into the [`Instance`] under construction.
    ///
    /// [`Store`]: struct.Store.html
    fn extract_tables(
        &self,
        mut context: impl AsContextMut,
        builder: &mut InstanceEntityBuilder,
    ) -> Result<(), InstantiationError> {
        let ctx = context.as_context_mut().store;
        if !ctx.can_create_more_tables(self.len_tables()) {
            return Err(InstantiationError::TooManyTables);
        }
        for table_type in self.internal_tables().copied() {
            let init = Val::default(table_type.element());
            let table =
                Table::new(context.as_context_mut(), table_type, init).map_err(|error| {
                    let error = match error.kind() {
                        ErrorKind::Table(error) => *error,
                        error => panic!("unexpected error: {error}"),
                    };
                    InstantiationError::FailedToInstantiateTable(error)
                })?;
            builder.push_table(table);
        }
        Ok(())
    }

    /// Extracts the Wasm linear memories from the module and stores them into the [`Store`].
    ///
    /// This also stores [`Memory`] references into the [`Instance`] under construction.
    ///
    /// [`Store`]: struct.Store.html
    fn extract_memories(
        &self,
        mut context: impl AsContextMut,
        builder: &mut InstanceEntityBuilder,
    ) -> Result<(), InstantiationError> {
        let ctx = context.as_context_mut().store;
        if !ctx.can_create_more_memories(self.len_memories()) {
            return Err(InstantiationError::TooManyMemories);
        }
        for memory_type in self.internal_memories().copied() {
            let memory = Memory::new(context.as_context_mut(), memory_type).map_err(|error| {
                let error = match error.kind() {
                    ErrorKind::Memory(error) => *error,
                    error => panic!("unexpected error: {error}"),
                };
                InstantiationError::FailedToInstantiateMemory(error)
            })?;
            builder.push_memory(memory);
        }
        Ok(())
    }

    /// Extracts the Wasm global variables from the module and stores them into the [`Store`].
    ///
    /// This also stores [`Global`] references into the [`Instance`] under construction.
    ///
    /// [`Store`]: struct.Store.html
    fn extract_globals(&self, mut context: impl AsContextMut, builder: &mut InstanceEntityBuilder) {
        for (global_type, global_init) in self.internal_globals() {
            let value_type = global_type.content();
            let init_value = Self::eval_init_expr(context.as_context_mut(), builder, global_init);
            let mutability = global_type.mutability();
            let global = Global::new(
                context.as_context_mut(),
                init_value.with_type(value_type),
                mutability,
            );
            builder.push_global(global);
        }
    }

    /// Evaluates the given initializer expression using the partially constructed [`Instance`].
    fn eval_init_expr(
        context: impl AsContext,
        builder: &InstanceEntityBuilder,
        init_expr: &ConstExpr,
    ) -> UntypedVal {
        init_expr
            .eval_with_context(
                |global_index| builder.get_global(global_index).get(&context),
                |func_index| <Ref<Func>>::from(builder.get_func(func_index)),
            )
            .expect("must evaluate to proper value")
    }

    /// Extracts the Wasm exports from the module and registers them into the [`Instance`].
    fn extract_exports(&self, builder: &mut InstanceEntityBuilder) {
        for (field, idx) in &self.module_header().exports {
            let external = match idx {
                export::ExternIdx::Func(func_index) => {
                    let func_index = func_index.into_u32();
                    let func = builder.get_func(func_index);
                    Extern::Func(func)
                }
                export::ExternIdx::Table(table_index) => {
                    let table_index = table_index.into_u32();
                    let table = builder.get_table(table_index);
                    Extern::Table(table)
                }
                export::ExternIdx::Memory(memory_index) => {
                    let memory_index = memory_index.into_u32();
                    let memory = builder.get_memory(memory_index);
                    Extern::Memory(memory)
                }
                export::ExternIdx::Global(global_index) => {
                    let global_index = global_index.into_u32();
                    let global = builder.get_global(global_index);
                    Extern::Global(global)
                }
            };
            builder.push_export(field, external);
        }
    }

    /// Extracts the optional start function for the build instance.
    fn extract_start_fn(&self, builder: &mut InstanceEntityBuilder) {
        if let Some(start_fn) = self.module_header().start {
            builder.set_start(start_fn)
        }
    }

    /// Initializes the [`Instance`] tables with the Wasm element segments of the [`Module`].
    fn initialize_table_elements(
        &self,
        mut context: impl AsContextMut,
        builder: &mut InstanceEntityBuilder,
    ) -> Result<(), Error> {
        for segment in &self.module_header().element_segments[..] {
            let get_global = |index| builder.get_global(index);
            let get_func = |index| builder.get_func(index);
            let element =
                ElementSegment::new(context.as_context_mut(), segment, get_func, get_global);
            if let ElementSegmentKind::Active(active) = segment.kind() {
                let dst_index = u64::from(Self::eval_init_expr(
                    context.as_context(),
                    builder,
                    active.offset(),
                ));
                let table = builder.get_table(active.table_index().into_u32());
                // Note: This checks not only that the elements in the element segments properly
                //       fit into the table at the given offset but also that the element segment
                //       consists of at least 1 element member.
                let len_table = table.size(&context);
                let len_items = element.size(&context);
                dst_index
                    .checked_add(u64::from(len_items))
                    .filter(|&max_index| max_index <= len_table)
                    .ok_or(InstantiationError::ElementSegmentDoesNotFit {
                        table,
                        table_index: dst_index,
                        len: len_items,
                    })?;
                let (table, elem) = context
                    .as_context_mut()
                    .store
                    .inner
                    .resolve_table_and_element_mut(&table, &element);
                table.init(elem.as_ref(), dst_index, 0, len_items, None)?;
                // Now drop the active element segment as commanded by the Wasm spec.
                elem.drop_items();
            }
            builder.push_element_segment(element);
        }
        Ok(())
    }

    /// Initializes the [`Instance`] linear memories with the Wasm data segments of the [`Module`].
    fn initialize_memory_data(
        &self,
        mut context: impl AsContextMut,
        builder: &mut InstanceEntityBuilder,
    ) -> Result<(), Error> {
        for segment in &self.inner.data_segments {
            let segment = match segment {
                InitDataSegment::Active {
                    memory_index,
                    offset,
                    bytes,
                } => {
                    let memory = builder.get_memory(memory_index.into_u32());
                    let offset = Self::eval_init_expr(context.as_context(), builder, offset);
                    let offset = match usize::try_from(u64::from(offset)) {
                        Ok(offset) => offset,
                        Err(_) => return Err(Error::from(MemoryError::OutOfBoundsAccess)),
                    };
                    memory.write(context.as_context_mut(), offset, bytes)?;
                    DataSegment::new_active(context.as_context_mut())
                }
                InitDataSegment::Passive { bytes } => {
                    DataSegment::new_passive(context.as_context_mut(), bytes)
                }
            };
            builder.push_data_segment(segment);
        }
        Ok(())
    }
}
