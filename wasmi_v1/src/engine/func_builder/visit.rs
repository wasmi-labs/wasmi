#![allow(unused_imports, unused_variables)] // TODO: remove

use super::{FuncValidator, FunctionBuilder, RelativeDepth, TranslationError};
use crate::{
    engine::bytecode::Instruction,
    module::{BlockType, FuncIdx, FuncTypeIdx, GlobalIdx, MemoryIdx, TableIdx},
};
use core::fmt::{self, Display};
use wasmparser::{BinaryReaderError, VisitOperator};

impl<'alloc, 'parser> FunctionBuilder<'alloc, 'parser> {
    /// Translates into `wasmi` bytecode if the current code path is reachable.
    ///
    /// # Note
    ///
    /// Ignores the `translator` closure if the current code path is unreachable.
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
    fn validate_then_translate_memory_op<F>(
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
}

impl<'alloc, 'parser> VisitOperator<'parser> for FunctionBuilder<'alloc, 'parser> {
    type Output = Result<(), TranslationError>;

    fn visit_unreachable(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate(|v| v.visit_unreachable(offset), Self::translate_unreachable)
    }

    fn visit_nop(&mut self, offset: usize) -> Self::Output {
        self.validator.visit_nop(offset)?;
        Ok(())
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
        self.validate_then_translate(|v| v.visit_else(offset), FunctionBuilder::translate_else)
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
        self.validate_then_translate(|v| v.visit_end(offset), |this| this.translate_end())
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
        self.validate_then_translate(
            |v| v.visit_return(offset),
            FunctionBuilder::translate_return,
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
        self.validator.visit_catch_all(offset).map_err(Into::into)
    }

    fn visit_drop(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate(|v| v.visit_drop(offset), |this| this.translate_drop())
    }

    fn visit_select(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate(|v| v.visit_select(offset), |this| this.translate_select())
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
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i32_load,
            FunctionBuilder::translate_i32_load,
        )
    }

    fn visit_i64_load(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i64_load,
            FunctionBuilder::translate_i64_load,
        )
    }

    fn visit_f32_load(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_f32_load,
            FunctionBuilder::translate_f32_load,
        )
    }

    fn visit_f64_load(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_f64_load,
            FunctionBuilder::translate_f64_load,
        )
    }

    fn visit_i32_load8_s(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i32_load8_s,
            FunctionBuilder::translate_i32_load8_s,
        )
    }

    fn visit_i32_load8_u(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i32_load8_u,
            FunctionBuilder::translate_i32_load8_u,
        )
    }

    fn visit_i32_load16_s(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i32_load16_s,
            FunctionBuilder::translate_i32_load16_s,
        )
    }

    fn visit_i32_load16_u(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i32_load16_u,
            FunctionBuilder::translate_i32_load16_u,
        )
    }

    fn visit_i64_load8_s(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i64_load8_s,
            FunctionBuilder::translate_i64_load8_s,
        )
    }

    fn visit_i64_load8_u(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i64_load8_u,
            FunctionBuilder::translate_i64_load8_u,
        )
    }

    fn visit_i64_load16_s(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i64_load16_s,
            FunctionBuilder::translate_i64_load16_s,
        )
    }

    fn visit_i64_load16_u(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i64_load16_u,
            FunctionBuilder::translate_i64_load16_u,
        )
    }

    fn visit_i64_load32_s(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i64_load32_s,
            FunctionBuilder::translate_i64_load32_s,
        )
    }

    fn visit_i64_load32_u(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i64_load32_u,
            FunctionBuilder::translate_i64_load32_u,
        )
    }

    fn visit_i32_store(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i32_store,
            FunctionBuilder::translate_i32_store,
        )
    }

    fn visit_i64_store(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i64_store,
            FunctionBuilder::translate_i64_store,
        )
    }

    fn visit_f32_store(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_f32_store,
            FunctionBuilder::translate_f32_store,
        )
    }

    fn visit_f64_store(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_f64_store,
            FunctionBuilder::translate_f64_store,
        )
    }

    fn visit_i32_store8(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i32_store8,
            FunctionBuilder::translate_i32_store8,
        )
    }

    fn visit_i32_store16(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i32_store16,
            FunctionBuilder::translate_i32_store16,
        )
    }

    fn visit_i64_store8(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i64_store8,
            FunctionBuilder::translate_i64_store8,
        )
    }

    fn visit_i64_store16(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i64_store16,
            FunctionBuilder::translate_i64_store16,
        )
    }

    fn visit_i64_store32(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        self.validate_then_translate_memory_op(
            offset,
            memarg,
            FuncValidator::visit_i64_store32,
            FunctionBuilder::translate_i64_store32,
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
            FunctionBuilder::translate_i32_eqz,
        )
    }

    fn visit_i32_eq(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_eq,
            FunctionBuilder::translate_i32_eq,
        )
    }

    fn visit_i32_ne(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_ne,
            FunctionBuilder::translate_i32_ne,
        )
    }

    fn visit_i32_lt_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_lt_s,
            FunctionBuilder::translate_i32_lt_s,
        )
    }

    fn visit_i32_lt_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_lt_u,
            FunctionBuilder::translate_i32_lt_u,
        )
    }

    fn visit_i32_gt_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_gt_s,
            FunctionBuilder::translate_i32_gt_s,
        )
    }

    fn visit_i32_gt_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_gt_u,
            FunctionBuilder::translate_i32_gt_u,
        )
    }

    fn visit_i32_le_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_le_s,
            FunctionBuilder::translate_i32_le_s,
        )
    }

    fn visit_i32_le_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_le_u,
            FunctionBuilder::translate_i32_le_u,
        )
    }

    fn visit_i32_ge_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_ge_s,
            FunctionBuilder::translate_i32_ge_s,
        )
    }

    fn visit_i32_ge_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_ge_u,
            FunctionBuilder::translate_i32_ge_u,
        )
    }

    fn visit_i64_eqz(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_eqz,
            FunctionBuilder::translate_i64_eqz,
        )
    }

    fn visit_i64_eq(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_eq,
            FunctionBuilder::translate_i64_eq,
        )
    }

    fn visit_i64_ne(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_ne,
            FunctionBuilder::translate_i64_ne,
        )
    }

    fn visit_i64_lt_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_lt_s,
            FunctionBuilder::translate_i64_lt_s,
        )
    }

    fn visit_i64_lt_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_lt_u,
            FunctionBuilder::translate_i64_lt_u,
        )
    }

    fn visit_i64_gt_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_gt_s,
            FunctionBuilder::translate_i64_gt_s,
        )
    }

    fn visit_i64_gt_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_gt_u,
            FunctionBuilder::translate_i64_gt_u,
        )
    }

    fn visit_i64_le_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_le_s,
            FunctionBuilder::translate_i64_le_s,
        )
    }

    fn visit_i64_le_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_le_u,
            FunctionBuilder::translate_i64_le_u,
        )
    }

    fn visit_i64_ge_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_ge_s,
            FunctionBuilder::translate_i64_ge_s,
        )
    }

    fn visit_i64_ge_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_ge_u,
            FunctionBuilder::translate_i64_ge_u,
        )
    }

    fn visit_f32_eq(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_eq,
            FunctionBuilder::translate_f32_eq,
        )
    }

    fn visit_f32_ne(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_ne,
            FunctionBuilder::translate_f32_ne,
        )
    }

    fn visit_f32_lt(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_lt,
            FunctionBuilder::translate_f32_lt,
        )
    }

    fn visit_f32_gt(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_gt,
            FunctionBuilder::translate_f32_gt,
        )
    }

    fn visit_f32_le(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_le,
            FunctionBuilder::translate_f32_le,
        )
    }

    fn visit_f32_ge(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_ge,
            FunctionBuilder::translate_f32_ge,
        )
    }

    fn visit_f64_eq(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_eq,
            FunctionBuilder::translate_f64_eq,
        )
    }

    fn visit_f64_ne(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_ne,
            FunctionBuilder::translate_f64_ne,
        )
    }

    fn visit_f64_lt(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_lt,
            FunctionBuilder::translate_f64_lt,
        )
    }

    fn visit_f64_gt(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_gt,
            FunctionBuilder::translate_f64_gt,
        )
    }

    fn visit_f64_le(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_le,
            FunctionBuilder::translate_f64_le,
        )
    }

    fn visit_f64_ge(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_ge,
            FunctionBuilder::translate_f64_ge,
        )
    }

    fn visit_i32_clz(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_clz,
            FunctionBuilder::translate_i32_clz,
        )
    }

    fn visit_i32_ctz(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_ctz,
            FunctionBuilder::translate_i32_ctz,
        )
    }

    fn visit_i32_popcnt(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_popcnt,
            FunctionBuilder::translate_i32_popcnt,
        )
    }

    fn visit_i32_add(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_add,
            FunctionBuilder::translate_i32_add,
        )
    }

    fn visit_i32_sub(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_sub,
            FunctionBuilder::translate_i32_sub,
        )
    }

    fn visit_i32_mul(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_mul,
            FunctionBuilder::translate_i32_mul,
        )
    }

    fn visit_i32_div_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_div_s,
            FunctionBuilder::translate_i32_div_s,
        )
    }

    fn visit_i32_div_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_div_u,
            FunctionBuilder::translate_i32_div_u,
        )
    }

    fn visit_i32_rem_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_rem_s,
            FunctionBuilder::translate_i32_rem_s,
        )
    }

    fn visit_i32_rem_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_rem_u,
            FunctionBuilder::translate_i32_rem_u,
        )
    }

    fn visit_i32_and(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_and,
            FunctionBuilder::translate_i32_and,
        )
    }

    fn visit_i32_or(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_or,
            FunctionBuilder::translate_i32_or,
        )
    }

    fn visit_i32_xor(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_xor,
            FunctionBuilder::translate_i32_xor,
        )
    }

    fn visit_i32_shl(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_shl,
            FunctionBuilder::translate_i32_shl,
        )
    }

    fn visit_i32_shr_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_shr_s,
            FunctionBuilder::translate_i32_shr_s,
        )
    }

    fn visit_i32_shr_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_shr_u,
            FunctionBuilder::translate_i32_shr_u,
        )
    }

    fn visit_i32_rotl(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_rotl,
            FunctionBuilder::translate_i32_rotl,
        )
    }

    fn visit_i32_rotr(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_rotr,
            FunctionBuilder::translate_i32_rotr,
        )
    }

    fn visit_i64_clz(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_clz,
            FunctionBuilder::translate_i64_clz,
        )
    }

    fn visit_i64_ctz(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_ctz,
            FunctionBuilder::translate_i64_ctz,
        )
    }

    fn visit_i64_popcnt(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_popcnt,
            FunctionBuilder::translate_i64_popcnt,
        )
    }

    fn visit_i64_add(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_add,
            FunctionBuilder::translate_i64_add,
        )
    }

    fn visit_i64_sub(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_sub,
            FunctionBuilder::translate_i64_sub,
        )
    }

    fn visit_i64_mul(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_mul,
            FunctionBuilder::translate_i64_mul,
        )
    }

    fn visit_i64_div_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_div_s,
            FunctionBuilder::translate_i64_div_s,
        )
    }

    fn visit_i64_div_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_div_u,
            FunctionBuilder::translate_i64_div_u,
        )
    }

    fn visit_i64_rem_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_rem_s,
            FunctionBuilder::translate_i64_rem_s,
        )
    }

    fn visit_i64_rem_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_rem_u,
            FunctionBuilder::translate_i64_rem_u,
        )
    }

    fn visit_i64_and(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_and,
            FunctionBuilder::translate_i64_and,
        )
    }

    fn visit_i64_or(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_or,
            FunctionBuilder::translate_i64_or,
        )
    }

    fn visit_i64_xor(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_xor,
            FunctionBuilder::translate_i64_xor,
        )
    }

    fn visit_i64_shl(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_shl,
            FunctionBuilder::translate_i64_shl,
        )
    }

    fn visit_i64_shr_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_shr_s,
            FunctionBuilder::translate_i64_shr_s,
        )
    }

    fn visit_i64_shr_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_shr_u,
            FunctionBuilder::translate_i64_shr_u,
        )
    }

    fn visit_i64_rotl(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_rotl,
            FunctionBuilder::translate_i64_rotl,
        )
    }

    fn visit_i64_rotr(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_rotr,
            FunctionBuilder::translate_i64_rotr,
        )
    }

    fn visit_f32_abs(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_abs,
            FunctionBuilder::translate_f32_abs,
        )
    }

    fn visit_f32_neg(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_neg,
            FunctionBuilder::translate_f32_neg,
        )
    }

    fn visit_f32_ceil(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_ceil,
            FunctionBuilder::translate_f32_ceil,
        )
    }

    fn visit_f32_floor(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_floor,
            FunctionBuilder::translate_f32_floor,
        )
    }

    fn visit_f32_trunc(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_trunc,
            FunctionBuilder::translate_f32_trunc,
        )
    }

    fn visit_f32_nearest(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_nearest,
            FunctionBuilder::translate_f32_nearest,
        )
    }

    fn visit_f32_sqrt(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_sqrt,
            FunctionBuilder::translate_f32_sqrt,
        )
    }

    fn visit_f32_add(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_add,
            FunctionBuilder::translate_f32_add,
        )
    }

    fn visit_f32_sub(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_sub,
            FunctionBuilder::translate_f32_sub,
        )
    }

    fn visit_f32_mul(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_mul,
            FunctionBuilder::translate_f32_mul,
        )
    }

    fn visit_f32_div(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_div,
            FunctionBuilder::translate_f32_div,
        )
    }

    fn visit_f32_min(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_min,
            FunctionBuilder::translate_f32_min,
        )
    }

    fn visit_f32_max(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_max,
            FunctionBuilder::translate_f32_max,
        )
    }

    fn visit_f32_copysign(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_copysign,
            FunctionBuilder::translate_f32_copysign,
        )
    }

    fn visit_f64_abs(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_abs,
            FunctionBuilder::translate_f64_abs,
        )
    }

    fn visit_f64_neg(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_neg,
            FunctionBuilder::translate_f64_neg,
        )
    }

    fn visit_f64_ceil(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_ceil,
            FunctionBuilder::translate_f64_ceil,
        )
    }

    fn visit_f64_floor(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_floor,
            FunctionBuilder::translate_f64_floor,
        )
    }

    fn visit_f64_trunc(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_trunc,
            FunctionBuilder::translate_f64_trunc,
        )
    }

    fn visit_f64_nearest(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_nearest,
            FunctionBuilder::translate_f64_nearest,
        )
    }

    fn visit_f64_sqrt(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_sqrt,
            FunctionBuilder::translate_f64_sqrt,
        )
    }

    fn visit_f64_add(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_add,
            FunctionBuilder::translate_f64_add,
        )
    }

    fn visit_f64_sub(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_sub,
            FunctionBuilder::translate_f64_sub,
        )
    }

    fn visit_f64_mul(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_mul,
            FunctionBuilder::translate_f64_mul,
        )
    }

    fn visit_f64_div(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_div,
            FunctionBuilder::translate_f64_div,
        )
    }

    fn visit_f64_min(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_min,
            FunctionBuilder::translate_f64_min,
        )
    }

    fn visit_f64_max(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_max,
            FunctionBuilder::translate_f64_max,
        )
    }

    fn visit_f64_copysign(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_copysign,
            FunctionBuilder::translate_f64_copysign,
        )
    }

    fn visit_i32_wrap_i64(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_wrap_i64,
            FunctionBuilder::translate_i32_wrap_i64,
        )
    }

    fn visit_i32_trunc_f32s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_trunc_f32s,
            FunctionBuilder::translate_i32_trunc_f32_s,
        )
    }

    fn visit_i32_trunc_f32u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_trunc_f32u,
            FunctionBuilder::translate_i32_trunc_f32_u,
        )
    }

    fn visit_i32_trunc_f64s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_trunc_f64s,
            FunctionBuilder::translate_i32_trunc_f64_s,
        )
    }

    fn visit_i32_trunc_f64u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_trunc_f64u,
            FunctionBuilder::translate_i32_trunc_f64_u,
        )
    }

    fn visit_i64_extend_i32s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_extend_i32s,
            FunctionBuilder::translate_i64_extend_i32_s,
        )
    }

    fn visit_i64_extend_i32u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_extend_i32u,
            FunctionBuilder::translate_i64_extend_i32_u,
        )
    }

    fn visit_i64_trunc_f32s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_trunc_f32s,
            FunctionBuilder::translate_i64_trunc_f32_s,
        )
    }

    fn visit_i64_trunc_f32u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_trunc_f32u,
            FunctionBuilder::translate_i64_trunc_f32_u,
        )
    }

    fn visit_i64_trunc_f64s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_trunc_f64s,
            FunctionBuilder::translate_i64_trunc_f64_s,
        )
    }

    fn visit_i64_trunc_f64u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_trunc_f64u,
            FunctionBuilder::translate_i64_trunc_f64_u,
        )
    }

    fn visit_f32_convert_i32s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_convert_i32s,
            FunctionBuilder::translate_f32_convert_i32_s,
        )
    }

    fn visit_f32_convert_i32u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_convert_i32u,
            FunctionBuilder::translate_f32_convert_i32_u,
        )
    }

    fn visit_f32_convert_i64s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_convert_i64s,
            FunctionBuilder::translate_f32_convert_i64_s,
        )
    }

    fn visit_f32_convert_i64u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_convert_i64u,
            FunctionBuilder::translate_f32_convert_i64_u,
        )
    }

    fn visit_f32_demote_f64(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_demote_f64,
            FunctionBuilder::translate_f32_demote_f64,
        )
    }

    fn visit_f64_convert_i32_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_convert_i32_s,
            FunctionBuilder::translate_f64_convert_i32_s,
        )
    }

    fn visit_f64_convert_i32_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_convert_i32_u,
            FunctionBuilder::translate_f64_convert_i32_u,
        )
    }

    fn visit_f64_convert_i64_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_convert_i64_s,
            FunctionBuilder::translate_f64_convert_i64_s,
        )
    }

    fn visit_f64_convert_i64_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_convert_i64_u,
            FunctionBuilder::translate_f64_convert_i64_u,
        )
    }

    fn visit_f64_promote_f32(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_promote_f32,
            FunctionBuilder::translate_f64_promote_f32,
        )
    }

    fn visit_i32_reinterpret_f32(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_reinterpret_f32,
            FunctionBuilder::translate_i32_reinterpret_f32,
        )
    }

    fn visit_i64_reinterpret_f64(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_reinterpret_f64,
            FunctionBuilder::translate_i64_reinterpret_f64,
        )
    }

    fn visit_f32_reinterpret_i32(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f32_reinterpret_i32,
            FunctionBuilder::translate_f32_reinterpret_i32,
        )
    }

    fn visit_f64_reinterpret_i64(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_f64_reinterpret_i64,
            FunctionBuilder::translate_f64_reinterpret_i64,
        )
    }

    fn visit_i32_extend8_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_extend8_s,
            FunctionBuilder::translate_i32_extend8_s,
        )
    }

    fn visit_i32_extend16_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_extend16_s,
            FunctionBuilder::translate_i32_extend16_s,
        )
    }

    fn visit_i64_extend8_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_extend8_s,
            FunctionBuilder::translate_i64_extend8_s,
        )
    }

    fn visit_i64_extend16_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_extend16_s,
            FunctionBuilder::translate_i64_extend16_s,
        )
    }

    fn visit_i64_extend32_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_extend32_s,
            FunctionBuilder::translate_i64_extend32_s,
        )
    }

    fn visit_i32_trunc_sat_f32_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_trunc_sat_f32_s,
            FunctionBuilder::translate_i32_trunc_sat_f32_s,
        )
    }

    fn visit_i32_trunc_sat_f32_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_trunc_sat_f32_u,
            FunctionBuilder::translate_i32_trunc_sat_f32_u,
        )
    }

    fn visit_i32_trunc_sat_f64_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_trunc_sat_f64_s,
            FunctionBuilder::translate_i32_trunc_sat_f64_s,
        )
    }

    fn visit_i32_trunc_sat_f64_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i32_trunc_sat_f64_u,
            FunctionBuilder::translate_i32_trunc_sat_f64_u,
        )
    }

    fn visit_i64_trunc_sat_f32_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_trunc_sat_f32_s,
            FunctionBuilder::translate_i64_trunc_sat_f32_s,
        )
    }

    fn visit_i64_trunc_sat_f32_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_trunc_sat_f32_u,
            FunctionBuilder::translate_i64_trunc_sat_f32_u,
        )
    }

    fn visit_i64_trunc_sat_f64_s(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_trunc_sat_f64_s,
            FunctionBuilder::translate_i64_trunc_sat_f64_s,
        )
    }

    fn visit_i64_trunc_sat_f64_u(&mut self, offset: usize) -> Self::Output {
        self.validate_then_translate_simple(
            offset,
            FuncValidator::visit_i64_trunc_sat_f64_u,
            FunctionBuilder::translate_i64_trunc_sat_f64_u,
        )
    }

    fn visit_memory_init(&mut self, offset: usize, segment: u32, mem: u32) -> Self::Output {
        todo!()
    }

    fn visit_data_drop(&mut self, offset: usize, segment: u32) -> Self::Output {
        todo!()
    }

    fn visit_memory_copy(&mut self, offset: usize, dst: u32, src: u32) -> Self::Output {
        todo!()
    }

    fn visit_memory_fill(&mut self, offset: usize, mem: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_init(&mut self, offset: usize, segment: u32, table: u32) -> Self::Output {
        todo!()
    }

    fn visit_elem_drop(&mut self, offset: usize, segment: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_copy(&mut self, offset: usize, dst_table: u32, src_table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_fill(&mut self, offset: usize, table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_get(&mut self, offset: usize, table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_set(&mut self, offset: usize, table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_grow(&mut self, offset: usize, table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_size(&mut self, offset: usize, table: u32) -> Self::Output {
        todo!()
    }

    fn visit_memory_atomic_notify(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_memory_atomic_wait32(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_memory_atomic_wait64(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_atomic_fence(&mut self, offset: usize, flags: u8) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_load(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_load(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_load8_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_load16_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_load8_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_load16_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_load32_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_store(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_store(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_store8(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_store16(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_store8(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_store16(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_store32(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw_add(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw_add(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw8_add_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw16_add_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw8_add_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw16_add_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw32_add_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw_sub(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw_sub(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw8_sub_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw16_sub_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw8_sub_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw16_sub_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw32_sub_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw_and(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw_and(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw8_and_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw16_and_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw8_and_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw16_and_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw32_and_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw_or(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw_or(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw8_or_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw16_or_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw8_or_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw16_or_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw32_or_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw_xor(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw_xor(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw8_xor_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw16_xor_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw8_xor_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw16_xor_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw32_xor_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw_xchg(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw_xchg(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw8_xchg_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw16_xchg_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw8_xchg_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw16_xchg_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw32_xchg_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw_cmpxchg(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw_cmpxchg(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw8_cmpxchg_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i32_atomic_rmw16_cmpxchg_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw8_cmpxchg_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw16_cmpxchg_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_i64_atomic_rmw32_cmpxchg_u(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_v128_load(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load8x8_s(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load8x8_u(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load16x4_s(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load16x4_u(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load32x2_s(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load32x2_u(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load8_splat(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_v128_load16_splat(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_v128_load32_splat(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_v128_load64_splat(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_v128_load32_zero(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_v128_load64_zero(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
    ) -> Self::Output {
        todo!()
    }

    fn visit_v128_store(&mut self, offset: usize, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load8_lane(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
        lane: u8,
    ) -> Self::Output {
        todo!()
    }

    fn visit_v128_load16_lane(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
        lane: u8,
    ) -> Self::Output {
        todo!()
    }

    fn visit_v128_load32_lane(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
        lane: u8,
    ) -> Self::Output {
        todo!()
    }

    fn visit_v128_load64_lane(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
        lane: u8,
    ) -> Self::Output {
        todo!()
    }

    fn visit_v128_store8_lane(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
        lane: u8,
    ) -> Self::Output {
        todo!()
    }

    fn visit_v128_store16_lane(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
        lane: u8,
    ) -> Self::Output {
        todo!()
    }

    fn visit_v128_store32_lane(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
        lane: u8,
    ) -> Self::Output {
        todo!()
    }

    fn visit_v128_store64_lane(
        &mut self,
        offset: usize,
        memarg: wasmparser::MemArg,
        lane: u8,
    ) -> Self::Output {
        todo!()
    }

    fn visit_v128_const(&mut self, offset: usize, value: wasmparser::V128) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_shuffle(&mut self, offset: usize, lanes: [u8; 16]) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_extract_lane_s(&mut self, offset: usize, lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_extract_lane_u(&mut self, offset: usize, lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_replace_lane(&mut self, offset: usize, lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extract_lane_s(&mut self, offset: usize, lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extract_lane_u(&mut self, offset: usize, lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_replace_lane(&mut self, offset: usize, lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extract_lane(&mut self, offset: usize, lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_replace_lane(&mut self, offset: usize, lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_extract_lane(&mut self, offset: usize, lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_replace_lane(&mut self, offset: usize, lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_extract_lane(&mut self, offset: usize, lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_replace_lane(&mut self, offset: usize, lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_extract_lane(&mut self, offset: usize, lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_replace_lane(&mut self, offset: usize, lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_swizzle(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_splat(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_splat(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_splat(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_splat(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_splat(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_splat(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_eq(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_ne(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_lt_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_lt_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_gt_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_gt_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_le_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_le_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_ge_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_ge_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_eq(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_ne(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_lt_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_lt_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_gt_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_gt_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_le_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_le_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_ge_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_ge_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_eq(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_ne(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_lt_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_lt_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_gt_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_gt_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_le_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_le_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_ge_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_ge_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_eq(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_ne(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_lt_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_gt_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_le_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_ge_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_eq(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_ne(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_lt(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_gt(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_le(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_ge(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_eq(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_ne(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_lt(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_gt(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_le(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_ge(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_v128_not(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_v128_and(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_v128_andnot(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_v128_or(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_v128_xor(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_v128_bitselect(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_v128_any_true(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_abs(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_neg(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_popcnt(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_all_true(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_bitmask(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_narrow_i16x8_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_narrow_i16x8_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_shl(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_shr_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_shr_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_add(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_add_sat_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_add_sat_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_sub(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_sub_sat_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_sub_sat_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_min_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_min_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_max_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_max_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_avgr_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extadd_pairwise_i8x16_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extadd_pairwise_i8x16_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_abs(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_neg(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_q15mulr_sat_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_all_true(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_bitmask(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_narrow_i32x4_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_narrow_i32x4_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extend_low_i8x16_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extend_high_i8x16_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extend_low_i8x16_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extend_high_i8x16_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_shl(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_shr_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_shr_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_add(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_add_sat_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_add_sat_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_sub(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_sub_sat_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_sub_sat_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_mul(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_min_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_min_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_max_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_max_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_avgr_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extmul_low_i8x16_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extmul_high_i8x16_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extmul_low_i8x16_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extmul_high_i8x16_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extadd_pairwise_i16x8_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extadd_pairwise_i16x8_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_abs(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_neg(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_all_true(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_bitmask(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extend_low_i16x8_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extend_high_i16x8_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extend_low_i16x8_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extend_high_i16x8_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_shl(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_shr_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_shr_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_add(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_sub(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_mul(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_min_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_min_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_max_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_max_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_dot_i16x8_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extmul_low_i16x8_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extmul_high_i16x8_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extmul_low_i16x8_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extmul_high_i16x8_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_abs(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_neg(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_all_true(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_bitmask(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_extend_low_i32x4_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_extend_high_i32x4_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_extend_low_i32x4_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_extend_high_i32x4_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_shl(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_shr_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_shr_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_add(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_sub(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_mul(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_extmul_low_i32x4_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_extmul_high_i32x4_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_extmul_low_i32x4_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_extmul_high_i32x4_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_ceil(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_floor(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_trunc(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_nearest(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_abs(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_neg(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_sqrt(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_add(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_sub(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_mul(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_div(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_min(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_max(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_pmin(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_pmax(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_ceil(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_floor(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_trunc(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_nearest(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_abs(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_neg(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_sqrt(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_add(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_sub(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_mul(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_div(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_min(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_max(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_pmin(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_pmax(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_trunc_sat_f32x4_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_trunc_sat_f32x4_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_convert_i32x4_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_convert_i32x4_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_trunc_sat_f64x2_s_zero(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_trunc_sat_f64x2_u_zero(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_convert_low_i32x4_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_convert_low_i32x4_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_demote_f64x2_zero(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_promote_low_f32x4(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_relaxed_swizzle(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_relaxed_trunc_sat_f32x4_s(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_relaxed_trunc_sat_f32x4_u(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_relaxed_trunc_sat_f64x2_s_zero(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_relaxed_trunc_sat_f64x2_u_zero(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_fma(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_fms(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_fma(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_fms(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_laneselect(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_laneselect(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_laneselect(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_laneselect(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_relaxed_min(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_relaxed_max(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_relaxed_min(&mut self, offset: usize) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_relaxed_max(&mut self, offset: usize) -> Self::Output {
        todo!()
    }
}
