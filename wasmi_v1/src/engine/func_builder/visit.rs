use super::{FuncBuilder, FuncValidator, RelativeDepth, TranslationError};
use crate::module::{BlockType, FuncIdx, FuncTypeIdx, GlobalIdx, MemoryIdx, TableIdx};
use wasmparser::{BinaryReaderError, MemArg, VisitOperator, V128};

impl<'alloc, 'parser> FuncBuilder<'alloc, 'parser> {
    /// Translates into `wasmi` bytecode if the current code path is reachable.
    fn validate_then_translate<V, F>(
        &mut self,
        validate: V,
        translator: F,
    ) -> Result<(), TranslationError>
    where
        V: FnOnce(&mut FuncValidator) -> Result<(), BinaryReaderError>,
        F: FnOnce(&mut Self) -> Result<(), TranslationError>,
    {
        validate(&mut self.validator)?;
        translator(self)?;
        Ok(())
    }

    /// Translates into `wasmi` bytecode if the current code path is reachable.
    ///
    /// # Note
    ///
    /// This is a simpler version than [`validate_then_translate`] and should
    /// be preferred if possible.
    ///
    /// [`validate_then_translate`]: [`Self::validate_then_translate`]
    fn validate_then_translate_simple(
        &mut self,
        offset: usize,
        validate: fn(&mut FuncValidator, usize) -> Result<(), BinaryReaderError>,
        translator: fn(&mut Self) -> Result<(), TranslationError>,
    ) -> Result<(), TranslationError> {
        validate(&mut self.validator, offset)?;
        translator(self)?;
        Ok(())
    }

    /// Decompose a [`wasmparser::MemArg`] into its raw parts.
    fn decompose_memarg(memarg: wasmparser::MemArg) -> (MemoryIdx, u32) {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        (memory_idx, offset)
    }

    /// Validate then translate a Wasm memory operator.
    fn validate_then_translate_memarg<F>(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
        validate: fn(
            &mut FuncValidator,
            usize,
            wasmparser::MemArg,
        ) -> Result<(), BinaryReaderError>,
        translate: F,
    ) -> Result<(), TranslationError>
    where
        F: FnOnce(&mut Self, MemoryIdx, u32) -> Result<(), TranslationError>,
    {
        self.validate_then_translate(
            |v| validate(v, offset, memarg),
            |this| {
                let (memory_idx, memory_offset) = Self::decompose_memarg(memarg);
                translate(this, memory_idx, memory_offset)
            },
        )
    }

    /// Forwards to the internal function validator.
    ///
    /// This is preferred over the more generic [`validate_or_err`] method
    /// if applicable.
    ///
    /// # Note
    ///
    /// This API is expected to be used if the function validator always
    /// returns an error. This is useful for unsupported Wasm proposals.
    ///
    /// [`validate_or_err`]: [`Self::validate_or_err`]
    fn validate_or_err_simple(
        &mut self,
        offset: usize,
        validate: fn(&mut FuncValidator, usize) -> Result<(), BinaryReaderError>,
    ) -> Result<(), TranslationError> {
        validate(&mut self.validator, offset).map_err(Into::into)
    }
}

macro_rules! define_unsupported_visit_operator {
    // The outer layer of repetition represents how all operators are
    // provided to the macro at the same time.
    //
    // The `$op` name is bound to the `Operator` variant name. The
    // payload of the operator is optionally specified (the `$(...)?`
    // clause) since not all instructions have payloads. Within the payload
    // each argument is named and has its type specified.
    //
    // The `$visit` name is bound to the corresponding name in the
    // `VisitOperator` trait that this corresponds to.
    ($( $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident)*) => {
        $(
            fn $visit(&mut self, offset: usize $($(,$arg: $argty)*)?) -> Self::Output {
                self.validator.$visit(offset $($(,$arg)*)?).map_err(Into::into)
            }
        )*
    }
}

impl<'alloc, 'parser> VisitOperator<'parser> for FuncBuilder<'alloc, 'parser> {
    type Output = Result<(), TranslationError>;

    for_each_unsupported_operator!(define_unsupported_visit_operator);

    fn visit_unreachable(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_unreachable,
            FuncBuilder::translate_unreachable,
        )
    }

    fn visit_nop(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(offset, FuncValidator::visit_nop, |_| Ok(()))
    }

    fn visit_block(&mut self, offset: usize, ty: wasmparser::BlockType) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_block(offset, ty),
            |this| {
                let ty = BlockType::try_from_wasmparser(ty, this.res)?;
                this.translate_block(ty)
            },
        )
    }

    fn visit_loop(&mut self, offset: usize, ty: wasmparser::BlockType) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_loop(offset, ty),
            |this| {
                let ty = BlockType::try_from_wasmparser(ty, this.res)?;
                this.translate_loop(ty)
            },
        )
    }

    fn visit_if(&mut self, offset: usize, ty: wasmparser::BlockType) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_if(offset, ty),
            |this| {
                let ty = BlockType::try_from_wasmparser(ty, this.res)?;
                this.translate_if(ty)
            },
        )
    }

    fn visit_else(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_else,
            FuncBuilder::translate_else,
        )
    }

    fn visit_try(&mut self, offset: usize, ty: wasmparser::BlockType) -> Self::Output {
        self.validator.visit_try(offset, ty).map_err(Into::into)
    }

    fn visit_catch(&mut self, offset: usize, index: u32) -> Self::Output {
        self.validator
            .visit_catch(offset, index)
            .map_err(Into::into)
    }

    fn visit_throw(&mut self, offset: usize, index: u32) -> Self::Output {
        self.validator
            .visit_throw(offset, index)
            .map_err(Into::into)
    }

    fn visit_rethrow(&mut self, offset: usize, relative_depth: u32) -> Self::Output {
        self.validator
            .visit_rethrow(offset, relative_depth)
            .map_err(Into::into)
    }

    fn visit_end(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_end,
            FuncBuilder::translate_end,
        )
    }

    fn visit_br(&mut self, offset: usize, relative_depth: u32) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_br(offset, relative_depth),
            |this| this.translate_br(relative_depth),
        )
    }

    fn visit_br_if(&mut self, offset: usize, relative_depth: u32) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_br_if(offset, relative_depth),
            |this| this.translate_br_if(relative_depth),
        )
    }

    fn visit_br_table(
        &mut self,
        offset: usize,
        table: wasmparser::BrTable<'parser>,
    ) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_br_table(offset, table.clone()),
            |this| {
                let default = RelativeDepth::from_u32(table.default());
                let targets = table
                    .targets()
                    .map(|relative_depth| {
                        relative_depth.unwrap_or_else(|error| {
                            panic!(
                                "encountered unexpected invalid relative depth for `br_table` target: {}",
                                error,
                            )
                        })
                    })
                    .map(RelativeDepth::from_u32);
                this.translate_br_table(default, targets)
            }
        )
    }

    fn visit_return(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_return,
            FuncBuilder::translate_return,
        )
    }

    fn visit_call(&mut self, offset: usize, function_index: u32) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_call(offset, function_index),
            |this| this.translate_call(FuncIdx(function_index)),
        )
    }

    fn visit_call_indirect(
        &mut self,
        offset: usize,
        index: u32,
        table_index: u32,
        table_byte: u8,
    ) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_call_indirect(offset, index, table_index, table_byte),
            |this| this.translate_call_indirect(FuncTypeIdx(index), TableIdx(table_index)),
        )
    }

    fn visit_return_call(&mut self, offset: usize, function_index: u32) -> Self::Output {
        self.validator
            .visit_return_call(offset, function_index)
            .map_err(Into::into)
    }

    fn visit_return_call_indirect(
        &mut self,
        offset: usize,
        index: u32,
        table_index: u32,
    ) -> Self::Output {
        self.validator
            .visit_return_call_indirect(offset, index, table_index)
            .map_err(Into::into)
    }

    fn visit_delegate(&mut self, offset: usize, relative_depth: u32) -> Self::Output {
        self.validator
            .visit_delegate(offset, relative_depth)
            .map_err(Into::into)
    }

    fn visit_catch_all(&mut self, offset: usize) -> Self::Output {
        self.validate_or_err_simple(offset, FuncValidator::visit_catch_all)
    }

    fn visit_drop(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_drop,
            FuncBuilder::translate_drop,
        )
    }

    fn visit_select(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_select,
            FuncBuilder::translate_select,
        )
    }

    fn visit_typed_select(&mut self, offset: usize, ty: wasmparser::ValType) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_typed_select(offset, ty),
            |this| this.translate_select(),
        )
    }

    fn visit_local_get(&mut self, offset: usize, local_index: u32) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_local_get(offset, local_index),
            |this| this.translate_local_get(local_index),
        )
    }

    fn visit_local_set(&mut self, offset: usize, local_index: u32) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_local_set(offset, local_index),
            |this| this.translate_local_set(local_index),
        )
    }

    fn visit_local_tee(&mut self, offset: usize, local_index: u32) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_local_tee(offset, local_index),
            |this| this.translate_local_tee(local_index),
        )
    }

    fn visit_global_get(&mut self, offset: usize, global_index: u32) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_global_get(offset, global_index),
            |this| this.translate_global_get(GlobalIdx(global_index)),
        )
    }

    fn visit_global_set(&mut self, offset: usize, global_index: u32) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_global_set(offset, global_index),
            |this| this.translate_global_set(GlobalIdx(global_index)),
        )
    }

    fn visit_i32_load(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i32_load,
            FuncBuilder::translate_i32_load,
        )
    }

    fn visit_i64_load(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i64_load,
            FuncBuilder::translate_i64_load,
        )
    }

    fn visit_f32_load(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_f32_load,
            FuncBuilder::translate_f32_load,
        )
    }

    fn visit_f64_load(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_f64_load,
            FuncBuilder::translate_f64_load,
        )
    }

    fn visit_i32_load8_s(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i32_load8_s,
            FuncBuilder::translate_i32_load8_s,
        )
    }

    fn visit_i32_load8_u(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i32_load8_u,
            FuncBuilder::translate_i32_load8_u,
        )
    }

    fn visit_i32_load16_s(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i32_load16_s,
            FuncBuilder::translate_i32_load16_s,
        )
    }

    fn visit_i32_load16_u(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i32_load16_u,
            FuncBuilder::translate_i32_load16_u,
        )
    }

    fn visit_i64_load8_s(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i64_load8_s,
            FuncBuilder::translate_i64_load8_s,
        )
    }

    fn visit_i64_load8_u(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i64_load8_u,
            FuncBuilder::translate_i64_load8_u,
        )
    }

    fn visit_i64_load16_s(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i64_load16_s,
            FuncBuilder::translate_i64_load16_s,
        )
    }

    fn visit_i64_load16_u(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i64_load16_u,
            FuncBuilder::translate_i64_load16_u,
        )
    }

    fn visit_i64_load32_s(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i64_load32_s,
            FuncBuilder::translate_i64_load32_s,
        )
    }

    fn visit_i64_load32_u(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i64_load32_u,
            FuncBuilder::translate_i64_load32_u,
        )
    }

    fn visit_i32_store(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i32_store,
            FuncBuilder::translate_i32_store,
        )
    }

    fn visit_i64_store(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i64_store,
            FuncBuilder::translate_i64_store,
        )
    }

    fn visit_f32_store(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_f32_store,
            FuncBuilder::translate_f32_store,
        )
    }

    fn visit_f64_store(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_f64_store,
            FuncBuilder::translate_f64_store,
        )
    }

    fn visit_i32_store8(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i32_store8,
            FuncBuilder::translate_i32_store8,
        )
    }

    fn visit_i32_store16(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i32_store16,
            FuncBuilder::translate_i32_store16,
        )
    }

    fn visit_i64_store8(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i64_store8,
            FuncBuilder::translate_i64_store8,
        )
    }

    fn visit_i64_store16(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i64_store16,
            FuncBuilder::translate_i64_store16,
        )
    }

    fn visit_i64_store32(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memarg(
            offset,
            memarg,
            FuncValidator::visit_i64_store32,
            FuncBuilder::translate_i64_store32,
        )
    }

    fn visit_memory_size(&mut self, offset: usize, mem: u32, mem_byte: u8) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_memory_size(offset, mem, mem_byte),
            |this| this.translate_memory_size(MemoryIdx(mem)),
        )
    }

    fn visit_memory_grow(&mut self, offset: usize, mem: u32, mem_byte: u8) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_memory_grow(offset, mem, mem_byte),
            |this| this.translate_memory_grow(MemoryIdx(mem)),
        )
    }

    fn visit_i32_const(&mut self, offset: usize, value: i32) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_i32_const(offset, value),
            |this| this.translate_i32_const(value),
        )
    }

    fn visit_i64_const(&mut self, offset: usize, value: i64) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_i64_const(offset, value),
            |this| this.translate_i64_const(value),
        )
    }

    fn visit_f32_const(&mut self, offset: usize, value: wasmparser::Ieee32) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_f32_const(offset, value),
            |this| this.translate_f32_const(value.bits().into()),
        )
    }

    fn visit_f64_const(&mut self, offset: usize, value: wasmparser::Ieee64) -> Self::Output {
        self.validate_then_translate(
            |v| v.visit_f64_const(offset, value),
            |this| this.translate_f64_const(value.bits().into()),
        )
    }

    fn visit_ref_null(&mut self, offset: usize, ty: wasmparser::ValType) -> Self::Output {
        self.validator
            .visit_ref_null(offset, ty)
            .map_err(Into::into)
    }

    fn visit_ref_is_null(&mut self, offset: usize) -> Self::Output {
        self.validator.visit_ref_is_null(offset).map_err(Into::into)
    }

    fn visit_ref_func(&mut self, offset: usize, function_index: u32) -> Self::Output {
        self.validator
            .visit_ref_func(offset, function_index)
            .map_err(Into::into)
    }

    fn visit_i32_eqz(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_eqz,
            FuncBuilder::translate_i32_eqz,
        )
    }

    fn visit_i32_eq(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_eq,
            FuncBuilder::translate_i32_eq,
        )
    }

    fn visit_i32_ne(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_ne,
            FuncBuilder::translate_i32_ne,
        )
    }

    fn visit_i32_lt_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_lt_s,
            FuncBuilder::translate_i32_lt_s,
        )
    }

    fn visit_i32_lt_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_lt_u,
            FuncBuilder::translate_i32_lt_u,
        )
    }

    fn visit_i32_gt_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_gt_s,
            FuncBuilder::translate_i32_gt_s,
        )
    }

    fn visit_i32_gt_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_gt_u,
            FuncBuilder::translate_i32_gt_u,
        )
    }

    fn visit_i32_le_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_le_s,
            FuncBuilder::translate_i32_le_s,
        )
    }

    fn visit_i32_le_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_le_u,
            FuncBuilder::translate_i32_le_u,
        )
    }

    fn visit_i32_ge_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_ge_s,
            FuncBuilder::translate_i32_ge_s,
        )
    }

    fn visit_i32_ge_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_ge_u,
            FuncBuilder::translate_i32_ge_u,
        )
    }

    fn visit_i64_eqz(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_eqz,
            FuncBuilder::translate_i64_eqz,
        )
    }

    fn visit_i64_eq(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_eq,
            FuncBuilder::translate_i64_eq,
        )
    }

    fn visit_i64_ne(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_ne,
            FuncBuilder::translate_i64_ne,
        )
    }

    fn visit_i64_lt_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_lt_s,
            FuncBuilder::translate_i64_lt_s,
        )
    }

    fn visit_i64_lt_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_lt_u,
            FuncBuilder::translate_i64_lt_u,
        )
    }

    fn visit_i64_gt_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_gt_s,
            FuncBuilder::translate_i64_gt_s,
        )
    }

    fn visit_i64_gt_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_gt_u,
            FuncBuilder::translate_i64_gt_u,
        )
    }

    fn visit_i64_le_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_le_s,
            FuncBuilder::translate_i64_le_s,
        )
    }

    fn visit_i64_le_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_le_u,
            FuncBuilder::translate_i64_le_u,
        )
    }

    fn visit_i64_ge_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_ge_s,
            FuncBuilder::translate_i64_ge_s,
        )
    }

    fn visit_i64_ge_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_ge_u,
            FuncBuilder::translate_i64_ge_u,
        )
    }

    fn visit_f32_eq(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_eq,
            FuncBuilder::translate_f32_eq,
        )
    }

    fn visit_f32_ne(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_ne,
            FuncBuilder::translate_f32_ne,
        )
    }

    fn visit_f32_lt(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_lt,
            FuncBuilder::translate_f32_lt,
        )
    }

    fn visit_f32_gt(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_gt,
            FuncBuilder::translate_f32_gt,
        )
    }

    fn visit_f32_le(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_le,
            FuncBuilder::translate_f32_le,
        )
    }

    fn visit_f32_ge(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_ge,
            FuncBuilder::translate_f32_ge,
        )
    }

    fn visit_f64_eq(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_eq,
            FuncBuilder::translate_f64_eq,
        )
    }

    fn visit_f64_ne(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_ne,
            FuncBuilder::translate_f64_ne,
        )
    }

    fn visit_f64_lt(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_lt,
            FuncBuilder::translate_f64_lt,
        )
    }

    fn visit_f64_gt(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_gt,
            FuncBuilder::translate_f64_gt,
        )
    }

    fn visit_f64_le(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_le,
            FuncBuilder::translate_f64_le,
        )
    }

    fn visit_f64_ge(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_ge,
            FuncBuilder::translate_f64_ge,
        )
    }

    fn visit_i32_clz(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_clz,
            FuncBuilder::translate_i32_clz,
        )
    }

    fn visit_i32_ctz(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_ctz,
            FuncBuilder::translate_i32_ctz,
        )
    }

    fn visit_i32_popcnt(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_popcnt,
            FuncBuilder::translate_i32_popcnt,
        )
    }

    fn visit_i32_add(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_add,
            FuncBuilder::translate_i32_add,
        )
    }

    fn visit_i32_sub(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_sub,
            FuncBuilder::translate_i32_sub,
        )
    }

    fn visit_i32_mul(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_mul,
            FuncBuilder::translate_i32_mul,
        )
    }

    fn visit_i32_div_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_div_s,
            FuncBuilder::translate_i32_div_s,
        )
    }

    fn visit_i32_div_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_div_u,
            FuncBuilder::translate_i32_div_u,
        )
    }

    fn visit_i32_rem_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_rem_s,
            FuncBuilder::translate_i32_rem_s,
        )
    }

    fn visit_i32_rem_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_rem_u,
            FuncBuilder::translate_i32_rem_u,
        )
    }

    fn visit_i32_and(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_and,
            FuncBuilder::translate_i32_and,
        )
    }

    fn visit_i32_or(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_or,
            FuncBuilder::translate_i32_or,
        )
    }

    fn visit_i32_xor(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_xor,
            FuncBuilder::translate_i32_xor,
        )
    }

    fn visit_i32_shl(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_shl,
            FuncBuilder::translate_i32_shl,
        )
    }

    fn visit_i32_shr_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_shr_s,
            FuncBuilder::translate_i32_shr_s,
        )
    }

    fn visit_i32_shr_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_shr_u,
            FuncBuilder::translate_i32_shr_u,
        )
    }

    fn visit_i32_rotl(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_rotl,
            FuncBuilder::translate_i32_rotl,
        )
    }

    fn visit_i32_rotr(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_rotr,
            FuncBuilder::translate_i32_rotr,
        )
    }

    fn visit_i64_clz(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_clz,
            FuncBuilder::translate_i64_clz,
        )
    }

    fn visit_i64_ctz(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_ctz,
            FuncBuilder::translate_i64_ctz,
        )
    }

    fn visit_i64_popcnt(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_popcnt,
            FuncBuilder::translate_i64_popcnt,
        )
    }

    fn visit_i64_add(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_add,
            FuncBuilder::translate_i64_add,
        )
    }

    fn visit_i64_sub(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_sub,
            FuncBuilder::translate_i64_sub,
        )
    }

    fn visit_i64_mul(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_mul,
            FuncBuilder::translate_i64_mul,
        )
    }

    fn visit_i64_div_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_div_s,
            FuncBuilder::translate_i64_div_s,
        )
    }

    fn visit_i64_div_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_div_u,
            FuncBuilder::translate_i64_div_u,
        )
    }

    fn visit_i64_rem_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_rem_s,
            FuncBuilder::translate_i64_rem_s,
        )
    }

    fn visit_i64_rem_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_rem_u,
            FuncBuilder::translate_i64_rem_u,
        )
    }

    fn visit_i64_and(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_and,
            FuncBuilder::translate_i64_and,
        )
    }

    fn visit_i64_or(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_or,
            FuncBuilder::translate_i64_or,
        )
    }

    fn visit_i64_xor(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_xor,
            FuncBuilder::translate_i64_xor,
        )
    }

    fn visit_i64_shl(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_shl,
            FuncBuilder::translate_i64_shl,
        )
    }

    fn visit_i64_shr_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_shr_s,
            FuncBuilder::translate_i64_shr_s,
        )
    }

    fn visit_i64_shr_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_shr_u,
            FuncBuilder::translate_i64_shr_u,
        )
    }

    fn visit_i64_rotl(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_rotl,
            FuncBuilder::translate_i64_rotl,
        )
    }

    fn visit_i64_rotr(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_rotr,
            FuncBuilder::translate_i64_rotr,
        )
    }

    fn visit_f32_abs(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_abs,
            FuncBuilder::translate_f32_abs,
        )
    }

    fn visit_f32_neg(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_neg,
            FuncBuilder::translate_f32_neg,
        )
    }

    fn visit_f32_ceil(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_ceil,
            FuncBuilder::translate_f32_ceil,
        )
    }

    fn visit_f32_floor(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_floor,
            FuncBuilder::translate_f32_floor,
        )
    }

    fn visit_f32_trunc(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_trunc,
            FuncBuilder::translate_f32_trunc,
        )
    }

    fn visit_f32_nearest(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_nearest,
            FuncBuilder::translate_f32_nearest,
        )
    }

    fn visit_f32_sqrt(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_sqrt,
            FuncBuilder::translate_f32_sqrt,
        )
    }

    fn visit_f32_add(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_add,
            FuncBuilder::translate_f32_add,
        )
    }

    fn visit_f32_sub(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_sub,
            FuncBuilder::translate_f32_sub,
        )
    }

    fn visit_f32_mul(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_mul,
            FuncBuilder::translate_f32_mul,
        )
    }

    fn visit_f32_div(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_div,
            FuncBuilder::translate_f32_div,
        )
    }

    fn visit_f32_min(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_min,
            FuncBuilder::translate_f32_min,
        )
    }

    fn visit_f32_max(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_max,
            FuncBuilder::translate_f32_max,
        )
    }

    fn visit_f32_copysign(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_copysign,
            FuncBuilder::translate_f32_copysign,
        )
    }

    fn visit_f64_abs(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_abs,
            FuncBuilder::translate_f64_abs,
        )
    }

    fn visit_f64_neg(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_neg,
            FuncBuilder::translate_f64_neg,
        )
    }

    fn visit_f64_ceil(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_ceil,
            FuncBuilder::translate_f64_ceil,
        )
    }

    fn visit_f64_floor(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_floor,
            FuncBuilder::translate_f64_floor,
        )
    }

    fn visit_f64_trunc(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_trunc,
            FuncBuilder::translate_f64_trunc,
        )
    }

    fn visit_f64_nearest(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_nearest,
            FuncBuilder::translate_f64_nearest,
        )
    }

    fn visit_f64_sqrt(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_sqrt,
            FuncBuilder::translate_f64_sqrt,
        )
    }

    fn visit_f64_add(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_add,
            FuncBuilder::translate_f64_add,
        )
    }

    fn visit_f64_sub(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_sub,
            FuncBuilder::translate_f64_sub,
        )
    }

    fn visit_f64_mul(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_mul,
            FuncBuilder::translate_f64_mul,
        )
    }

    fn visit_f64_div(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_div,
            FuncBuilder::translate_f64_div,
        )
    }

    fn visit_f64_min(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_min,
            FuncBuilder::translate_f64_min,
        )
    }

    fn visit_f64_max(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_max,
            FuncBuilder::translate_f64_max,
        )
    }

    fn visit_f64_copysign(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_copysign,
            FuncBuilder::translate_f64_copysign,
        )
    }

    fn visit_i32_wrap_i64(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_wrap_i64,
            FuncBuilder::translate_i32_wrap_i64,
        )
    }

    fn visit_i32_trunc_f32s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_trunc_f32s,
            FuncBuilder::translate_i32_trunc_f32_s,
        )
    }

    fn visit_i32_trunc_f32u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_trunc_f32u,
            FuncBuilder::translate_i32_trunc_f32_u,
        )
    }

    fn visit_i32_trunc_f64s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_trunc_f64s,
            FuncBuilder::translate_i32_trunc_f64_s,
        )
    }

    fn visit_i32_trunc_f64u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_trunc_f64u,
            FuncBuilder::translate_i32_trunc_f64_u,
        )
    }

    fn visit_i64_extend_i32s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_extend_i32s,
            FuncBuilder::translate_i64_extend_i32_s,
        )
    }

    fn visit_i64_extend_i32u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_extend_i32u,
            FuncBuilder::translate_i64_extend_i32_u,
        )
    }

    fn visit_i64_trunc_f32s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_trunc_f32s,
            FuncBuilder::translate_i64_trunc_f32_s,
        )
    }

    fn visit_i64_trunc_f32u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_trunc_f32u,
            FuncBuilder::translate_i64_trunc_f32_u,
        )
    }

    fn visit_i64_trunc_f64s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_trunc_f64s,
            FuncBuilder::translate_i64_trunc_f64_s,
        )
    }

    fn visit_i64_trunc_f64u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_trunc_f64u,
            FuncBuilder::translate_i64_trunc_f64_u,
        )
    }

    fn visit_f32_convert_i32s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_convert_i32s,
            FuncBuilder::translate_f32_convert_i32_s,
        )
    }

    fn visit_f32_convert_i32u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_convert_i32u,
            FuncBuilder::translate_f32_convert_i32_u,
        )
    }

    fn visit_f32_convert_i64s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_convert_i64s,
            FuncBuilder::translate_f32_convert_i64_s,
        )
    }

    fn visit_f32_convert_i64u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_convert_i64u,
            FuncBuilder::translate_f32_convert_i64_u,
        )
    }

    fn visit_f32_demote_f64(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_demote_f64,
            FuncBuilder::translate_f32_demote_f64,
        )
    }

    fn visit_f64_convert_i32_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_convert_i32_s,
            FuncBuilder::translate_f64_convert_i32_s,
        )
    }

    fn visit_f64_convert_i32_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_convert_i32_u,
            FuncBuilder::translate_f64_convert_i32_u,
        )
    }

    fn visit_f64_convert_i64_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_convert_i64_s,
            FuncBuilder::translate_f64_convert_i64_s,
        )
    }

    fn visit_f64_convert_i64_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_convert_i64_u,
            FuncBuilder::translate_f64_convert_i64_u,
        )
    }

    fn visit_f64_promote_f32(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_promote_f32,
            FuncBuilder::translate_f64_promote_f32,
        )
    }

    fn visit_i32_reinterpret_f32(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_reinterpret_f32,
            FuncBuilder::translate_i32_reinterpret_f32,
        )
    }

    fn visit_i64_reinterpret_f64(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_reinterpret_f64,
            FuncBuilder::translate_i64_reinterpret_f64,
        )
    }

    fn visit_f32_reinterpret_i32(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_reinterpret_i32,
            FuncBuilder::translate_f32_reinterpret_i32,
        )
    }

    fn visit_f64_reinterpret_i64(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_reinterpret_i64,
            FuncBuilder::translate_f64_reinterpret_i64,
        )
    }

    fn visit_i32_extend8_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_extend8_s,
            FuncBuilder::translate_i32_extend8_s,
        )
    }

    fn visit_i32_extend16_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_extend16_s,
            FuncBuilder::translate_i32_extend16_s,
        )
    }

    fn visit_i64_extend8_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_extend8_s,
            FuncBuilder::translate_i64_extend8_s,
        )
    }

    fn visit_i64_extend16_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_extend16_s,
            FuncBuilder::translate_i64_extend16_s,
        )
    }

    fn visit_i64_extend32_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_extend32_s,
            FuncBuilder::translate_i64_extend32_s,
        )
    }

    fn visit_i32_trunc_sat_f32_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_trunc_sat_f32_s,
            FuncBuilder::translate_i32_trunc_sat_f32_s,
        )
    }

    fn visit_i32_trunc_sat_f32_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_trunc_sat_f32_u,
            FuncBuilder::translate_i32_trunc_sat_f32_u,
        )
    }

    fn visit_i32_trunc_sat_f64_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_trunc_sat_f64_s,
            FuncBuilder::translate_i32_trunc_sat_f64_s,
        )
    }

    fn visit_i32_trunc_sat_f64_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_trunc_sat_f64_u,
            FuncBuilder::translate_i32_trunc_sat_f64_u,
        )
    }

    fn visit_i64_trunc_sat_f32_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_trunc_sat_f32_s,
            FuncBuilder::translate_i64_trunc_sat_f32_s,
        )
    }

    fn visit_i64_trunc_sat_f32_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_trunc_sat_f32_u,
            FuncBuilder::translate_i64_trunc_sat_f32_u,
        )
    }

    fn visit_i64_trunc_sat_f64_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_trunc_sat_f64_s,
            FuncBuilder::translate_i64_trunc_sat_f64_s,
        )
    }

    fn visit_i64_trunc_sat_f64_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_trunc_sat_f64_u,
            FuncBuilder::translate_i64_trunc_sat_f64_u,
        )
    }
}
