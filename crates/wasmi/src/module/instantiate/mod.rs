mod error;
mod pre;

#[cfg(test)]
mod tests;

pub use self::{error::InstantiationError, pre::InstancePre};
use super::{element::ElementSegmentKind, export, ConstExpr, DataSegmentKind, Module};
use crate::{
    func::WasmFuncEntity,
    memory::{DataSegment, MemoryError},
    value::WithType,
    AsContext,
    AsContextMut,
    ElementSegment,
    Error,
    Extern,
    ExternType,
    FuncRef,
    FuncType,
    Global,
    Instance,
    InstanceEntity,
    InstanceEntityBuilder,
    Memory,
    Table,
    Value,
};
use wasmi_core::{Trap, UntypedValue};

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
    pub(crate) fn instantiate<I>(
        &self,
        mut context: impl AsContextMut,
        externals: I,
    ) -> Result<InstancePre, Error>
    where
        I: IntoIterator<Item = Extern>,
    {
        context
            .as_context_mut()
            .store
            .check_new_instances_limit(1)?;
        let handle = context.as_context_mut().store.inner.alloc_instance();
        let mut builder = InstanceEntity::build(self);

        self.extract_imports(&mut context, &mut builder, externals)?;
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
            match (import.ty(), external) {
                (ExternType::Func(expected_signature), Extern::Func(func)) => {
                    let actual_signature = func.ty_dedup(context.as_context());
                    let actual_signature = self
                        .engine
                        .resolve_func_type(actual_signature, FuncType::clone);
                    // Note: We can compare function signatures without resolving them because
                    //       we deduplicate them before registering. Therefore two equal instances of
                    //       [`SignatureEntity`] will be associated to the same [`Signature`].
                    if &actual_signature != expected_signature {
                        // Note: In case of error we could resolve the signatures for better error readability.
                        return Err(InstantiationError::SignatureMismatch {
                            actual: actual_signature,
                            expected: expected_signature.clone(),
                        });
                    }
                    builder.push_func(func);
                }
                (ExternType::Table(required), Extern::Table(table)) => {
                    let imported = table.dynamic_ty(context.as_context());
                    imported.is_subtype_or_err(required)?;
                    builder.push_table(table);
                }
                (ExternType::Memory(required), Extern::Memory(memory)) => {
                    let imported = memory.dynamic_ty(context.as_context());
                    imported.is_subtype_or_err(required)?;
                    builder.push_memory(memory);
                }
                (ExternType::Global(required), Extern::Global(global)) => {
                    let imported = global.ty(context.as_context());
                    required.satisfies(&imported)?;
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
        context: &mut impl AsContextMut,
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
        context: &mut impl AsContextMut,
        builder: &mut InstanceEntityBuilder,
    ) -> Result<(), InstantiationError> {
        context
            .as_context_mut()
            .store
            .check_new_tables_limit(self.len_tables())?;
        for table_type in self.internal_tables().copied() {
            let init = Value::default(table_type.element());
            let table = Table::new(context.as_context_mut(), table_type, init)?;
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
        context: &mut impl AsContextMut,
        builder: &mut InstanceEntityBuilder,
    ) -> Result<(), MemoryError> {
        context
            .as_context_mut()
            .store
            .check_new_memories_limit(self.len_memories())?;
        for memory_type in self.internal_memories().copied() {
            let memory = Memory::new(context.as_context_mut(), memory_type)?;
            builder.push_memory(memory);
        }
        Ok(())
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
    ) -> UntypedValue {
        init_expr
            .eval_with_context(
                |global_index| builder.get_global(global_index).get(&context),
                |func_index| FuncRef::new(builder.get_func(func_index)),
            )
            .expect("must evaluate to proper value")
    }

    /// Extracts the Wasm exports from the module and registers them into the [`Instance`].
    fn extract_exports(&self, builder: &mut InstanceEntityBuilder) {
        for (field, idx) in &self.exports {
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
        if let Some(start_fn) = self.start {
            builder.set_start(start_fn)
        }
    }

    /// Initializes the [`Instance`] tables with the Wasm element segments of the [`Module`].
    fn initialize_table_elements(
        &self,
        mut context: &mut impl AsContextMut,
        builder: &mut InstanceEntityBuilder,
    ) -> Result<(), Error> {
        for segment in &self.element_segments[..] {
            let element = ElementSegment::new(context.as_context_mut(), segment);
            if let ElementSegmentKind::Active(active) = segment.kind() {
                let dst_index = u32::from(Self::eval_init_expr(
                    &mut *context,
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
                    .checked_add(len_items)
                    .filter(|&max_index| max_index <= len_table)
                    .ok_or(InstantiationError::ElementSegmentDoesNotFit {
                        table,
                        offset: dst_index,
                        amount: len_items,
                    })?;
                // Finally do the actual initialization of the table elements.
                {
                    let (table, element) = context
                        .as_context_mut()
                        .store
                        .inner
                        .resolve_table_element(&table, &element);
                    table
                        .init(dst_index, element, 0, len_items, |func_index| {
                            builder.get_func(func_index)
                        })
                        .map_err(Trap::from)?;
                }
                // Now drop the active element segment as commanded by the Wasm spec.
                element.drop_items(&mut context);
            }
            builder.push_element_segment(element);
        }
        Ok(())
    }

    /// Initializes the [`Instance`] linear memories with the Wasm data segments of the [`Module`].
    fn initialize_memory_data(
        &self,
        context: &mut impl AsContextMut,
        builder: &mut InstanceEntityBuilder,
    ) -> Result<(), Error> {
        for segment in &self.data_segments[..] {
            let bytes = segment.bytes();
            if let DataSegmentKind::Active(segment) = segment.kind() {
                let offset_expr = segment.offset();
                let offset =
                    u32::from(Self::eval_init_expr(&mut *context, builder, offset_expr)) as usize;
                let memory = builder.get_memory(segment.memory_index().into_u32());
                memory.write(&mut *context, offset, bytes)?;
            }
            builder.push_data_segment(DataSegment::new(context.as_context_mut(), segment));
        }
        Ok(())
    }
}
