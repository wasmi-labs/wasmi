use crate::{
    core::Trap,
    engine::{CallParams, CallResults, FuncBody},
    module::{FuncIdx, ModuleResources},
    AsContextMut,
    Engine,
    Func,
};
use core::fmt::{Debug, Display};

/// The used function validator type.
#[allow(dead_code)] // TODO: remove
type FuncValidator = wasmparser::FuncValidator<wasmparser::ValidatorResources>;

/// A `wasmi` engine backend.
pub trait EngineBackend {
    /// The WebAssembly function translator of the backend.
    type Translator<'parser>: TranslateWasmFunc<'parser>;
}

/// A type that can execute a compiled Wasm function.
pub trait ExecuteWasmFunc {
    /// Executes the Wasm function `func` given the `params`.
    ///
    /// Stores the results back into `results` or returns a [`Trap`].
    fn execute_func<Params, Results>(
        &mut self,
        ctx: impl AsContextMut,
        func: Func,
        params: Params,
        results: Results,
    ) -> Result<<Results as CallResults>::Results, Trap>
    where
        Params: CallParams,
        Results: CallResults;
}

/// A type that can translate Wasm function into its own IR.
pub trait TranslateWasmFunc<'parser> {
    /// An engine specific translation error.
    type Error: Debug + Display + Send;

    /// Reusable allocations of a Wasm function translator.
    ///
    /// These allocations can be reused across multiple function translations.
    /// They are primarily an optimization to avoid unnecessary heap allocations.
    type Allocations: Default + Sized;

    /// Creates a new Wasm translator for a given Wasm function `func`.
    ///
    /// # Params
    ///
    /// - `func`: The index of the Wasm function that is to be translated.
    /// - `res`: Provides important information about the associated Wasm module.
    /// - `validator`: A Wasm validator handle to validate the Wasm function
    ///                during its translation. Combining these steps is more
    ///                efficient than doing both in isolation.
    /// - `allocations`: Allocations and data required by the Wasm translator
    ///                  that can be reused across multiple translations.
    fn new(
        engine: &Engine,
        func: FuncIdx,
        res: ModuleResources<'parser>,
        validator: FuncValidator,
        allocations: Self::Allocations,
    ) -> Self;

    /// Registers the Wasm local variables for the translated function.
    ///
    /// # Note
    ///
    /// This method is required to be called before calling any of the
    /// `translate_` methods of the Wasm function translator.
    ///
    /// # Errors
    ///
    /// If more local variables are being registered than can be handled by the
    /// Wasm function translator.
    fn register_locals(
        &mut self,
        offset: usize,
        amount: u32,
        value_type: wasmparser::ValType,
    ) -> Result<(), Self::Error>;

    /// Finishes translation of the Wasm function.
    ///
    /// Returns both a reference to the resulting Wasm function body as well
    /// as the used Wasm function translator allocations that can be reused
    /// for the next Wasm function translation.
    fn finish(self, offset: usize) -> Result<(FuncBody, Self::Allocations), Self::Error>;

    fn translate_nop(&mut self) -> Result<(), Self::Error>;
    fn translate_unreachable(&mut self) -> Result<(), Self::Error>;
    fn translate_block(&mut self, block_type: wasmparser::BlockType) -> Result<(), Self::Error>;
    fn translate_loop(&mut self, block_type: wasmparser::BlockType) -> Result<(), Self::Error>;
    fn translate_if(&mut self, block_type: wasmparser::BlockType) -> Result<(), Self::Error>;
    fn translate_else(&mut self) -> Result<(), Self::Error>;
    fn translate_end(&mut self) -> Result<(), Self::Error>;
    fn translate_br(&mut self, relative_depth: u32) -> Result<(), Self::Error>;
    fn translate_br_if(&mut self, relative_depth: u32) -> Result<(), Self::Error>;
    fn translate_br_table(
        &mut self,
        table: wasmparser::BrTable<'parser>,
    ) -> Result<(), Self::Error>;
    fn translate_return(&mut self) -> Result<(), Self::Error>;
    fn translate_call(&mut self, func_idx: u32) -> Result<(), Self::Error>;
    fn translate_call_indirect(
        &mut self,
        index: u32,
        table_index: u32,
        _table_byte: u8,
    ) -> Result<(), Self::Error>;
    fn translate_drop(&mut self) -> Result<(), Self::Error>;
    fn translate_select(&mut self) -> Result<(), Self::Error>;
    fn translate_local_get(&mut self, local_idx: u32) -> Result<(), Self::Error>;
    fn translate_local_set(&mut self, local_idx: u32) -> Result<(), Self::Error>;
    fn translate_local_tee(&mut self, local_idx: u32) -> Result<(), Self::Error>;
    fn translate_global_get(&mut self, global_idx: u32) -> Result<(), Self::Error>;
    fn translate_global_set(&mut self, global_idx: u32) -> Result<(), Self::Error>;
    fn translate_i32_load(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_i64_load(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_f32_load(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_f64_load(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_i32_load8_s(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_i32_load8_u(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_i32_load16_s(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_i32_load16_u(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_i64_load8_s(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_i64_load8_u(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_i64_load16_s(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_i64_load16_u(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_i64_load32_s(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_i64_load32_u(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_i32_store(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_i64_store(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_f32_store(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_f64_store(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_i32_store8(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_i32_store16(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_i64_store8(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_i64_store16(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_i64_store32(&mut self, memarg: wasmparser::MemArg) -> Result<(), Self::Error>;
    fn translate_memory_size(&mut self, memory_idx: u32, _mem_byte: u8) -> Result<(), Self::Error>;
    fn translate_memory_grow(&mut self, memory_idx: u32, _mem_byte: u8) -> Result<(), Self::Error>;
    fn translate_i32_const(&mut self, value: i32) -> Result<(), Self::Error>;
    fn translate_i64_const(&mut self, value: i64) -> Result<(), Self::Error>;
    fn translate_f32_const(&mut self, value: wasmparser::Ieee32) -> Result<(), Self::Error>;
    fn translate_f64_const(&mut self, value: wasmparser::Ieee64) -> Result<(), Self::Error>;
    fn translate_i32_eqz(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_eq(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_ne(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_lt_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_lt_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_gt_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_gt_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_le_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_le_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_ge_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_ge_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_eqz(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_eq(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_ne(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_lt_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_lt_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_gt_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_gt_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_le_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_le_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_ge_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_ge_u(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_eq(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_ne(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_lt(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_gt(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_le(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_ge(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_eq(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_ne(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_lt(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_gt(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_le(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_ge(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_clz(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_ctz(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_popcnt(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_add(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_sub(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_mul(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_div_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_div_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_rem_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_rem_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_and(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_or(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_xor(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_shl(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_shr_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_shr_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_rotl(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_rotr(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_clz(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_ctz(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_popcnt(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_add(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_sub(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_mul(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_div_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_div_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_rem_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_rem_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_and(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_or(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_xor(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_shl(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_shr_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_shr_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_rotl(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_rotr(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_abs(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_neg(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_ceil(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_floor(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_trunc(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_nearest(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_sqrt(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_add(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_sub(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_mul(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_div(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_min(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_max(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_copysign(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_abs(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_neg(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_ceil(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_floor(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_trunc(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_nearest(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_sqrt(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_add(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_sub(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_mul(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_div(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_min(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_max(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_copysign(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_wrap_i64(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_trunc_f32_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_trunc_f32_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_trunc_f64_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_trunc_f64_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_extend_i32_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_extend_i32_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_trunc_f32_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_trunc_f32_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_trunc_f64_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_trunc_f64_u(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_convert_i32_s(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_convert_i32_u(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_convert_i64_s(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_convert_i64_u(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_demote_f64(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_convert_i32_s(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_convert_i32_u(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_convert_i64_s(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_convert_i64_u(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_promote_f32(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_reinterpret_f32(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_reinterpret_f64(&mut self) -> Result<(), Self::Error>;
    fn translate_f32_reinterpret_i32(&mut self) -> Result<(), Self::Error>;
    fn translate_f64_reinterpret_i64(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_extend8_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_extend16_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_extend8_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_extend16_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_extend32_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_trunc_sat_f32_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_trunc_sat_f32_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_trunc_sat_f64_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i32_trunc_sat_f64_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_trunc_sat_f32_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_trunc_sat_f32_u(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_trunc_sat_f64_s(&mut self) -> Result<(), Self::Error>;
    fn translate_i64_trunc_sat_f64_u(&mut self) -> Result<(), Self::Error>;
}
