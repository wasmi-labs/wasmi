use alloc::boxed::Box;

macro_rules! for_each_wasm_operator {
    ($mac:ident) => {
        $mac! {
            // Wasm MVP
            Unreachable { snake: unreachable }
            Nop { snake: nop }
            Block { snake: block }
            Loop { snake: loop_ }
            If { snake: if_ }
            Else { snake: else_ }
            End { snake: end }
            Br { snake: br }
            BrIf { snake: br_if }
            BrTable { snake: br_table }
            Return { snake: return_ }
            Call { snake: call }
            CallIndirect { snake: call_indirect }
            Drop { snake: drop }
            Select { snake: select }
            LocalGet { snake: local_get }
            LocalSet { snake: local_set }
            LocalTee { snake: local_tee }
            GlobalGet { snake: global_get }
            GlobalSet { snake: global_set }
            I32Load { snake: i32_load }
            I64Load { snake: i64_load }
            F32Load { snake: f32_load }
            F64Load { snake: f64_load }
            I32Load8S { snake: i32_load8_s }
            I32Load8U { snake: i32_load8_u }
            I32Load16S { snake: i32_load16_s }
            I32Load16U { snake: i32_load16_u }
            I64Load8S { snake: i64_load8_s }
            I64Load8U { snake: i64_load8_u }
            I64Load16S { snake: i64_load16_s }
            I64Load16U { snake: i64_load16_u }
            I64Load32S { snake: i64_load32_s }
            I64Load32U { snake: i64_load32_u }
            I32Store { snake: i32_store }
            I64Store { snake: i64_store }
            F32Store { snake: f32_store }
            F64Store { snake: f64_store }
            I32Store8 { snake: i32_store8 }
            I32Store16 { snake: i32_store16 }
            I64Store8 { snake: i64_store8 }
            I64Store16 { snake: i64_store16 }
            I64Store32 { snake: i64_store32 }
            MemorySize { snake: memory_size }
            MemoryGrow { snake: memory_grow }
            I32Const { snake: i32_const }
            I64Const { snake: i64_const }
            F32Const { snake: f32_const }
            F64Const { snake: f64_const }
            I32Eqz { snake: i32_eqz }
            I32Eq { snake: i32_eq }
            I32Ne { snake: i32_ne }
            I32LtS { snake: i32_lt_s }
            I32LtU { snake: i32_lt_u }
            I32GtS { snake: i32_gt_s }
            I32GtU { snake: i32_gt_u }
            I32LeS { snake: i32_le_s }
            I32LeU { snake: i32_le_u }
            I32GeS { snake: i32_ge_s }
            I32GeU { snake: i32_ge_u }
            I64Eqz { snake: i64_eqz }
            I64Eq { snake: i64_eq }
            I64Ne { snake: i64_ne }
            I64LtS { snake: i64_lt_s }
            I64LtU { snake: i64_lt_u }
            I64GtS { snake: i64_gt_s }
            I64GtU { snake: i64_gt_u }
            I64LeS { snake: i64_le_s }
            I64LeU { snake: i64_le_u }
            I64GeS { snake: i64_ge_s }
            I64GeU { snake: i64_ge_u }
            F32Eq { snake: f32_eq }
            F32Ne { snake: f32_ne }
            F32Lt { snake: f32_lt }
            F32Gt { snake: f32_gt }
            F32Le { snake: f32_le }
            F32Ge { snake: f32_ge }
            F64Eq { snake: f64_eq }
            F64Ne { snake: f64_ne }
            F64Lt { snake: f64_lt }
            F64Gt { snake: f64_gt }
            F64Le { snake: f64_le }
            F64Ge { snake: f64_ge }
            I32Clz { snake: i32_clz }
            I32Ctz { snake: i32_ctz }
            I32Popcnt { snake: i32_popcnt }
            I32Add { snake: i32_add }
            I32Sub { snake: i32_sub }
            I32Mul { snake: i32_mul }
            I32DivS { snake: i32_div_s }
            I32DivU { snake: i32_div_u }
            I32RemS { snake: i32_rem_s }
            I32RemU { snake: i32_rem_u }
            I32And { snake: i32_and }
            I32Or { snake: i32_or }
            I32Xor { snake: i32_xor }
            I32Shl { snake: i32_shl }
            I32ShrS { snake: i32_shr_s }
            I32ShrU { snake: i32_shr_u }
            I32Rotl { snake: i32_rotl }
            I32Rotr { snake: i32_rotr }
            I64Clz { snake: i64_clz }
            I64Ctz { snake: i64_ctz }
            I64Popcnt { snake: i64_popcnt }
            I64Add { snake: i64_add }
            I64Sub { snake: i64_sub }
            I64Mul { snake: i64_mul }
            I64DivS { snake: i64_div_s }
            I64DivU { snake: i64_div_u }
            I64RemS { snake: i64_rem_s }
            I64RemU { snake: i64_rem_u }
            I64And { snake: i64_and }
            I64Or { snake: i64_or }
            I64Xor { snake: i64_xor }
            I64Shl { snake: i64_shl }
            I64ShrS { snake: i64_shr_s }
            I64ShrU { snake: i64_shr_u }
            I64Rotl { snake: i64_rotl }
            I64Rotr { snake: i64_rotr }
            F32Abs { snake: f32_abs }
            F32Neg { snake: f32_neg }
            F32Ceil { snake: f32_ceil }
            F32Floor { snake: f32_floor }
            F32Trunc { snake: f32_trunc }
            F32Nearest { snake: f32_nearest }
            F32Sqrt { snake: f32_sqrt }
            F32Add { snake: f32_add }
            F32Sub { snake: f32_sub }
            F32Mul { snake: f32_mul }
            F32Div { snake: f32_div }
            F32Min { snake: f32_min }
            F32Max { snake: f32_max }
            F32Copysign { snake: f32_copysign }
            F64Abs { snake: f64_abs }
            F64Neg { snake: f64_neg }
            F64Ceil { snake: f64_ceil }
            F64Floor { snake: f64_floor }
            F64Trunc { snake: f64_trunc }
            F64Nearest { snake: f64_nearest }
            F64Sqrt { snake: f64_sqrt }
            F64Add { snake: f64_add }
            F64Sub { snake: f64_sub }
            F64Mul { snake: f64_mul }
            F64Div { snake: f64_div }
            F64Min { snake: f64_min }
            F64Max { snake: f64_max }
            F64Copysign { snake: f64_copysign }
            I32WrapI64 { snake: i32_wrap_i64 }
            I32TruncF32S { snake: i32_trunc_f32_s }
            I32TruncF32U { snake: i32_trunc_f32_u }
            I32TruncF64S { snake: i32_trunc_f64_s }
            I32TruncF64U { snake: i32_trunc_f64_u }
            I64ExtendI32S { snake: i64_extend_i32_s }
            I64ExtendI32U { snake: i64_extend_i32_u }
            I64TruncF32S { snake: i64_trunc_f32_s }
            I64TruncF32U { snake: i64_trunc_f32_u }
            I64TruncF64S { snake: i64_trunc_f64_s }
            I64TruncF64U { snake: i64_trunc_f64_u }
            F32ConvertI32S { snake: f32_convert_i32_s }
            F32ConvertI32U { snake: f32_convert_i32_u }
            F32ConvertI64S { snake: f32_convert_i64_s }
            F32ConvertI64U { snake: f32_convert_i64_u }
            F32DemoteF64 { snake: f32_demote_f64 }
            F64ConvertI32S { snake: f64_convert_i32_s }
            F64ConvertI32U { snake: f64_convert_i32_u }
            F64ConvertI64S { snake: f64_convert_i64_s }
            F64ConvertI64U { snake: f64_convert_i64_u }
            F64PromoteF32 { snake: f64_promote_f32 }
            I32ReinterpretF32 { snake: i32_reinterpret_f32 }
            I64ReinterpretF64 { snake: i64_reinterpret_f64 }
            F32ReinterpretI32 { snake: f32_reinterpret_i32 }
            F64ReinterpretI64 { snake: f64_reinterpret_i64 }

            // sign-extension
            I32Extend8S { snake: i32_extend8_s }
            I32Extend16S { snake: i32_extend16_s }
            I64Extend8S { snake: i64_extend8_s }
            I64Extend16S { snake: i64_extend16_s }
            I64Extend32S { snake: i64_extend32_s }

            // saturating f2i conversions
            I32TruncSatF32S { snake: i32_trunc_sat_f32_s }
            I32TruncSatF32U { snake: i32_trunc_sat_f32_u }
            I32TruncSatF64S { snake: i32_trunc_sat_f64_s }
            I32TruncSatF64U { snake: i32_trunc_sat_f64_u }
            I64TruncSatF32S { snake: i64_trunc_sat_f32_s }
            I64TruncSatF32U { snake: i64_trunc_sat_f32_u }
            I64TruncSatF64S { snake: i64_trunc_sat_f64_s }
            I64TruncSatF64U { snake: i64_trunc_sat_f64_u }

            // reference-types
            RefNull { snake: ref_null }
            RefIsNull { snake: ref_is_null }
            RefFunc { snake: ref_func }
            TypedSelect { snake: typed_select }

            // tail-call
            ReturnCall { snake: return_call }
            ReturnCallIndirect { snake: return_call_indirect }

            // bulk-ops
            MemoryInit { snake: memory_init }
            DataDrop { snake: data_drop }
            MemoryCopy { snake: memory_copy }
            MemoryFill { snake: memory_fill }
            TableInit { snake: table_init }
            ElemDrop { snake: elem_drop }
            TableCopy { snake: table_copy }
            TableFill { snake: table_fill }
            TableGet { snake: table_get }
            TableSet { snake: table_set }
            TableGrow { snake: table_grow }
            TableSize { snake: table_size }

            // wide-arithmetic
            I64Add128 { snake: i64_add128 }
            I64Sub128 { snake: i64_sub128 }
            I64MulWideS { snake: i64_mul_wide_s }
            I64MulWideU { snake: i64_mul_wide_u }
        }
    };
}

macro_rules! define_wasm_operator {
    ( $($camel:ident { snake: $snake:ident } )* ) => {
        /// A Wasm operator supported by Wasmi.
        #[derive(Debug, Copy, Clone)]
        #[non_exhaustive]
        pub enum WasmOperator {
            $( $camel ),*
        }
    };
}
for_each_wasm_operator!(define_wasm_operator);

macro_rules! default_cost {
    // Nop and drop generate no code, so don't consume fuel for them.
    (Nop) => {
        0
    };
    (Drop) => {
        0
    };
    // Control flow may create branches, but is generally cheap and
    // free, so don't consume fuel. Note the lack of `if` since some
    // cost is incurred with the conditional check.
    (Block) => {
        0
    };
    (Loop) => {
        0
    };
    (Unreachable) => {
        0
    };
    (Return) => {
        0
    };
    (Else) => {
        0
    };
    (End) => {
        0
    };
    // Everything else, just call it one operation.
    ($op:ident) => {
        1
    };
}

macro_rules! define_operator_cost {
    ( $($camel:ident { snake: $snake:ident } )* ) => {
        /// The fuel cost of each operator in a table.
        #[derive(Debug, Copy, Clone)]
        pub struct OperatorCost {
            $(
                pub $snake: u8
            ),*
        }

        impl Default for OperatorCost {
            fn default() -> Self {
                Self {
                    $( $snake: default_cost!($camel) ),*
                }
            }
        }

        impl OperatorCost {
            /// Returns the cost for `op`.
            #[inline]
            pub fn cost(&self, op: WasmOperator) -> u64 {
                let cost = match op {
                    $( WasmOperator::$camel => self.$snake ),*
                };
                u64::from(cost)
            }
        }
    };
}
for_each_wasm_operator!(define_operator_cost);

/// The strategy for fuel metering of Wasm operators.
#[derive(Debug, Default, Clone)]
pub enum OperatorCostStrategy {
    /// Use default Wasm operator costs defined by Wasmi.
    #[default]
    Default,
    /// Use custom Wasm operator costs defined by the user.
    Table(Box<OperatorCost>),
}

impl OperatorCostStrategy {
    /// Creates a new [`OperatorCostStrategy`] from the given table of Wasm operator costs.
    pub fn table(cost: OperatorCost) -> Self {
        Self::Table(Box::new(cost))
    }

    /// Returns the cost defined by `self` for `op`.
    pub fn cost(&self, op: WasmOperator) -> u64 {
        match self {
            Self::Default => Self::default_cost(op),
            Self::Table(table) => table.cost(op),
        }
    }
}

macro_rules! impl_operator_cost_strategy {
    ( $($camel:ident { snake: $snake:ident } )* ) => {
        impl OperatorCostStrategy {
            /// Returns Wasmi's default costs for `op`.
            fn default_cost(op: WasmOperator) -> u64 {
                match op {
                    $( WasmOperator::$camel => default_cost!($camel) ),*
                }
            }
        }
    };
}
for_each_wasm_operator!(impl_operator_cost_strategy);
