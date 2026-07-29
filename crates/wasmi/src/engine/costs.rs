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

            // simd
            V128Load { snake: v128_load, feature = "simd" }
            V128Load8x8S { snake: v128_load8x8_s, feature = "simd" }
            V128Load8x8U { snake: v128_load8x8_u, feature = "simd" }
            V128Load16x4S { snake: v128_load16x4_s, feature = "simd" }
            V128Load16x4U { snake: v128_load16x4_u, feature = "simd" }
            V128Load32x2S { snake: v128_load32x2_s, feature = "simd" }
            V128Load32x2U { snake: v128_load32x2_u, feature = "simd" }
            V128Load8Splat { snake: v128_load8_splat, feature = "simd" }
            V128Load16Splat { snake: v128_load16_splat, feature = "simd" }
            V128Load32Splat { snake: v128_load32_splat, feature = "simd" }
            V128Load64Splat { snake: v128_load64_splat, feature = "simd" }
            V128Load32Zero { snake: v128_load32_zero, feature = "simd" }
            V128Load64Zero { snake: v128_load64_zero, feature = "simd" }
            V128Store { snake: v128_store, feature = "simd" }
            V128Load8Lane { snake: v128_load8_lane, feature = "simd" }
            V128Load16Lane { snake: v128_load16_lane, feature = "simd" }
            V128Load32Lane { snake: v128_load32_lane, feature = "simd" }
            V128Load64Lane { snake: v128_load64_lane, feature = "simd" }
            V128Store8Lane { snake: v128_store8_lane, feature = "simd" }
            V128Store16Lane { snake: v128_store16_lane, feature = "simd" }
            V128Store32Lane { snake: v128_store32_lane, feature = "simd" }
            V128Store64Lane { snake: v128_store64_lane, feature = "simd" }
            V128Const { snake: v128_const, feature = "simd" }
            I8x16Shuffle { snake: i8x16_shuffle, feature = "simd" }
            I8x16ExtractLaneS { snake: i8x16_extract_lane_s, feature = "simd" }
            I8x16ExtractLaneU { snake: i8x16_extract_lane_u, feature = "simd" }
            I8x16ReplaceLane { snake: i8x16_replace_lane, feature = "simd" }
            I16x8ExtractLaneS { snake: i16x8_extract_lane_s, feature = "simd" }
            I16x8ExtractLaneU { snake: i16x8_extract_lane_u, feature = "simd" }
            I16x8ReplaceLane { snake: i16x8_replace_lane, feature = "simd" }
            I32x4ExtractLane { snake: i32x4_extract_lane, feature = "simd" }
            I32x4ReplaceLane { snake: i32x4_replace_lane, feature = "simd" }
            I64x2ExtractLane { snake: i64x2_extract_lane, feature = "simd" }
            I64x2ReplaceLane { snake: i64x2_replace_lane, feature = "simd" }
            F32x4ExtractLane { snake: f32x4_extract_lane, feature = "simd" }
            F32x4ReplaceLane { snake: f32x4_replace_lane, feature = "simd" }
            F64x2ExtractLane { snake: f64x2_extract_lane, feature = "simd" }
            F64x2ReplaceLane { snake: f64x2_replace_lane, feature = "simd" }
            I8x16Swizzle { snake: i8x16_swizzle, feature = "simd" }
            I8x16Splat { snake: i8x16_splat, feature = "simd" }
            I16x8Splat { snake: i16x8_splat, feature = "simd" }
            I32x4Splat { snake: i32x4_splat, feature = "simd" }
            I64x2Splat { snake: i64x2_splat, feature = "simd" }
            F32x4Splat { snake: f32x4_splat, feature = "simd" }
            F64x2Splat { snake: f64x2_splat, feature = "simd" }
            I8x16Eq { snake: i8x16_eq, feature = "simd" }
            I8x16Ne { snake: i8x16_ne, feature = "simd" }
            I8x16LtS { snake: i8x16_lt_s, feature = "simd" }
            I8x16LtU { snake: i8x16_lt_u, feature = "simd" }
            I8x16GtS { snake: i8x16_gt_s, feature = "simd" }
            I8x16GtU { snake: i8x16_gt_u, feature = "simd" }
            I8x16LeS { snake: i8x16_le_s, feature = "simd" }
            I8x16LeU { snake: i8x16_le_u, feature = "simd" }
            I8x16GeS { snake: i8x16_ge_s, feature = "simd" }
            I8x16GeU { snake: i8x16_ge_u, feature = "simd" }
            I16x8Eq { snake: i16x8_eq, feature = "simd" }
            I16x8Ne { snake: i16x8_ne, feature = "simd" }
            I16x8LtS { snake: i16x8_lt_s, feature = "simd" }
            I16x8LtU { snake: i16x8_lt_u, feature = "simd" }
            I16x8GtS { snake: i16x8_gt_s, feature = "simd" }
            I16x8GtU { snake: i16x8_gt_u, feature = "simd" }
            I16x8LeS { snake: i16x8_le_s, feature = "simd" }
            I16x8LeU { snake: i16x8_le_u, feature = "simd" }
            I16x8GeS { snake: i16x8_ge_s, feature = "simd" }
            I16x8GeU { snake: i16x8_ge_u, feature = "simd" }
            I32x4Eq { snake: i32x4_eq, feature = "simd" }
            I32x4Ne { snake: i32x4_ne, feature = "simd" }
            I32x4LtS { snake: i32x4_lt_s, feature = "simd" }
            I32x4LtU { snake: i32x4_lt_u, feature = "simd" }
            I32x4GtS { snake: i32x4_gt_s, feature = "simd" }
            I32x4GtU { snake: i32x4_gt_u, feature = "simd" }
            I32x4LeS { snake: i32x4_le_s, feature = "simd" }
            I32x4LeU { snake: i32x4_le_u, feature = "simd" }
            I32x4GeS { snake: i32x4_ge_s, feature = "simd" }
            I32x4GeU { snake: i32x4_ge_u, feature = "simd" }
            I64x2Eq { snake: i64x2_eq, feature = "simd" }
            I64x2Ne { snake: i64x2_ne, feature = "simd" }
            I64x2LtS { snake: i64x2_lt_s, feature = "simd" }
            I64x2GtS { snake: i64x2_gt_s, feature = "simd" }
            I64x2LeS { snake: i64x2_le_s, feature = "simd" }
            I64x2GeS { snake: i64x2_ge_s, feature = "simd" }
            F32x4Eq { snake: f32x4_eq, feature = "simd" }
            F32x4Ne { snake: f32x4_ne, feature = "simd" }
            F32x4Lt { snake: f32x4_lt, feature = "simd" }
            F32x4Gt { snake: f32x4_gt, feature = "simd" }
            F32x4Le { snake: f32x4_le, feature = "simd" }
            F32x4Ge { snake: f32x4_ge, feature = "simd" }
            F64x2Eq { snake: f64x2_eq, feature = "simd" }
            F64x2Ne { snake: f64x2_ne, feature = "simd" }
            F64x2Lt { snake: f64x2_lt, feature = "simd" }
            F64x2Gt { snake: f64x2_gt, feature = "simd" }
            F64x2Le { snake: f64x2_le, feature = "simd" }
            F64x2Ge { snake: f64x2_ge, feature = "simd" }
            V128Not { snake: v128_not, feature = "simd" }
            V128And { snake: v128_and, feature = "simd" }
            V128AndNot { snake: v128_and_not, feature = "simd" }
            V128Or { snake: v128_or, feature = "simd" }
            V128Xor { snake: v128_xor, feature = "simd" }
            V128Bitselect { snake: v128_bitselect, feature = "simd" }
            V128AnyTrue { snake: v128_any_true, feature = "simd" }
            I8x16Abs { snake: i8x16_abs, feature = "simd" }
            I8x16Neg { snake: i8x16_neg, feature = "simd" }
            I8x16Popcnt { snake: i8x16_popcnt, feature = "simd" }
            I8x16AllTrue { snake: i8x16_all_true, feature = "simd" }
            I8x16Bitmask { snake: i8x16_bitmask, feature = "simd" }
            I8x16NarrowI16x8S { snake: i8x16_narrow_i16x8_s, feature = "simd" }
            I8x16NarrowI16x8U { snake: i8x16_narrow_i16x8_u, feature = "simd" }
            I8x16Shl { snake: i8x16_shl, feature = "simd" }
            I8x16ShrS { snake: i8x16_shr_s, feature = "simd" }
            I8x16ShrU { snake: i8x16_shr_u, feature = "simd" }
            I8x16Add { snake: i8x16_add, feature = "simd" }
            I8x16AddSatS { snake: i8x16_add_sat_s, feature = "simd" }
            I8x16AddSatU { snake: i8x16_add_sat_u, feature = "simd" }
            I8x16Sub { snake: i8x16_sub, feature = "simd" }
            I8x16SubSatS { snake: i8x16_sub_sat_s, feature = "simd" }
            I8x16SubSatU { snake: i8x16_sub_sat_u, feature = "simd" }
            I8x16MinS { snake: i8x16_min_s, feature = "simd" }
            I8x16MinU { snake: i8x16_min_u, feature = "simd" }
            I8x16MaxS { snake: i8x16_max_s, feature = "simd" }
            I8x16MaxU { snake: i8x16_max_u, feature = "simd" }
            I8x16AvgrU { snake: i8x16_avgr_u, feature = "simd" }
            I16x8ExtAddPairwiseI8x16S { snake: i16x8_ext_add_pairwise_i8x16_s, feature = "simd" }
            I16x8ExtAddPairwiseI8x16U { snake: i16x8_ext_add_pairwise_i8x16_u, feature = "simd" }
            I16x8Abs { snake: i16x8_abs, feature = "simd" }
            I16x8Neg { snake: i16x8_neg, feature = "simd" }
            I16x8Q15MulrSatS { snake: i16x8_q15_mulr_sat_s, feature = "simd" }
            I16x8AllTrue { snake: i16x8_all_true, feature = "simd" }
            I16x8Bitmask { snake: i16x8_bitmask, feature = "simd" }
            I16x8NarrowI32x4S { snake: i16x8_narrow_i32x4_s, feature = "simd" }
            I16x8NarrowI32x4U { snake: i16x8_narrow_i32x4_u, feature = "simd" }
            I16x8ExtendLowI8x16S { snake: i16x8_extend_low_i8x16_s, feature = "simd" }
            I16x8ExtendHighI8x16S { snake: i16x8_extend_high_i8x16_s, feature = "simd" }
            I16x8ExtendLowI8x16U { snake: i16x8_extend_low_i8x16_u, feature = "simd" }
            I16x8ExtendHighI8x16U { snake: i16x8_extend_high_i8x16_u, feature = "simd" }
            I16x8Shl { snake: i16x8_shl, feature = "simd" }
            I16x8ShrS { snake: i16x8_shr_s, feature = "simd" }
            I16x8ShrU { snake: i16x8_shr_u, feature = "simd" }
            I16x8Add { snake: i16x8_add, feature = "simd" }
            I16x8AddSatS { snake: i16x8_add_sat_s, feature = "simd" }
            I16x8AddSatU { snake: i16x8_add_sat_u, feature = "simd" }
            I16x8Sub { snake: i16x8_sub, feature = "simd" }
            I16x8SubSatS { snake: i16x8_sub_sat_s, feature = "simd" }
            I16x8SubSatU { snake: i16x8_sub_sat_u, feature = "simd" }
            I16x8Mul { snake: i16x8_mul, feature = "simd" }
            I16x8MinS { snake: i16x8_min_s, feature = "simd" }
            I16x8MinU { snake: i16x8_min_u, feature = "simd" }
            I16x8MaxS { snake: i16x8_max_s, feature = "simd" }
            I16x8MaxU { snake: i16x8_max_u, feature = "simd" }
            I16x8AvgrU { snake: i16x8_avgr_u, feature = "simd" }
            I16x8ExtMulLowI8x16S { snake: i16x8_ext_mul_low_i8x16_s, feature = "simd" }
            I16x8ExtMulHighI8x16S { snake: i16x8_ext_mul_high_i8x16_s, feature = "simd" }
            I16x8ExtMulLowI8x16U { snake: i16x8_ext_mul_low_i8x16_u, feature = "simd" }
            I16x8ExtMulHighI8x16U { snake: i16x8_ext_mul_high_i8x16_u, feature = "simd" }
            I32x4ExtAddPairwiseI16x8S { snake: i32x4_ext_add_pairwise_i16x8_s, feature = "simd" }
            I32x4ExtAddPairwiseI16x8U { snake: i32x4_ext_add_pairwise_i16x8_u, feature = "simd" }
            I32x4Abs { snake: i32x4_abs, feature = "simd" }
            I32x4Neg { snake: i32x4_neg, feature = "simd" }
            I32x4AllTrue { snake: i32x4_all_true, feature = "simd" }
            I32x4Bitmask { snake: i32x4_bitmask, feature = "simd" }
            I32x4ExtendLowI16x8S { snake: i32x4_extend_low_i16x8_s, feature = "simd" }
            I32x4ExtendHighI16x8S { snake: i32x4_extend_high_i16x8_s, feature = "simd" }
            I32x4ExtendLowI16x8U { snake: i32x4_extend_low_i16x8_u, feature = "simd" }
            I32x4ExtendHighI16x8U { snake: i32x4_extend_high_i16x8_u, feature = "simd" }
            I32x4Shl { snake: i32x4_shl, feature = "simd" }
            I32x4ShrS { snake: i32x4_shr_s, feature = "simd" }
            I32x4ShrU { snake: i32x4_shr_u, feature = "simd" }
            I32x4Add { snake: i32x4_add, feature = "simd" }
            I32x4Sub { snake: i32x4_sub, feature = "simd" }
            I32x4Mul { snake: i32x4_mul, feature = "simd" }
            I32x4MinS { snake: i32x4_min_s, feature = "simd" }
            I32x4MinU { snake: i32x4_min_u, feature = "simd" }
            I32x4MaxS { snake: i32x4_max_s, feature = "simd" }
            I32x4MaxU { snake: i32x4_max_u, feature = "simd" }
            I32x4DotI16x8S { snake: i32x4_dot_i16x8_s, feature = "simd" }
            I32x4ExtMulLowI16x8S { snake: i32x4_ext_mul_low_i16x8_s, feature = "simd" }
            I32x4ExtMulHighI16x8S { snake: i32x4_ext_mul_high_i16x8_s, feature = "simd" }
            I32x4ExtMulLowI16x8U { snake: i32x4_ext_mul_low_i16x8_u, feature = "simd" }
            I32x4ExtMulHighI16x8U { snake: i32x4_ext_mul_high_i16x8_u, feature = "simd" }
            I64x2Abs { snake: i64x2_abs, feature = "simd" }
            I64x2Neg { snake: i64x2_neg, feature = "simd" }
            I64x2AllTrue { snake: i64x2_all_true, feature = "simd" }
            I64x2Bitmask { snake: i64x2_bitmask, feature = "simd" }
            I64x2ExtendLowI32x4S { snake: i64x2_extend_low_i32x4_s, feature = "simd" }
            I64x2ExtendHighI32x4S { snake: i64x2_extend_high_i32x4_s, feature = "simd" }
            I64x2ExtendLowI32x4U { snake: i64x2_extend_low_i32x4_u, feature = "simd" }
            I64x2ExtendHighI32x4U { snake: i64x2_extend_high_i32x4_u, feature = "simd" }
            I64x2Shl { snake: i64x2_shl, feature = "simd" }
            I64x2ShrS { snake: i64x2_shr_s, feature = "simd" }
            I64x2ShrU { snake: i64x2_shr_u, feature = "simd" }
            I64x2Add { snake: i64x2_add, feature = "simd" }
            I64x2Sub { snake: i64x2_sub, feature = "simd" }
            I64x2Mul { snake: i64x2_mul, feature = "simd" }
            I64x2ExtMulLowI32x4S { snake: i64x2_ext_mul_low_i32x4_s, feature = "simd" }
            I64x2ExtMulHighI32x4S { snake: i64x2_ext_mul_high_i32x4_s, feature = "simd" }
            I64x2ExtMulLowI32x4U { snake: i64x2_ext_mul_low_i32x4_u, feature = "simd" }
            I64x2ExtMulHighI32x4U { snake: i64x2_ext_mul_high_i32x4_u, feature = "simd" }
            F32x4Ceil { snake: f32x4_ceil, feature = "simd" }
            F32x4Floor { snake: f32x4_floor, feature = "simd" }
            F32x4Trunc { snake: f32x4_trunc, feature = "simd" }
            F32x4Nearest { snake: f32x4_nearest, feature = "simd" }
            F32x4Abs { snake: f32x4_abs, feature = "simd" }
            F32x4Neg { snake: f32x4_neg, feature = "simd" }
            F32x4Sqrt { snake: f32x4_sqrt, feature = "simd" }
            F32x4Add { snake: f32x4_add, feature = "simd" }
            F32x4Sub { snake: f32x4_sub, feature = "simd" }
            F32x4Mul { snake: f32x4_mul, feature = "simd" }
            F32x4Div { snake: f32x4_div, feature = "simd" }
            F32x4Min { snake: f32x4_min, feature = "simd" }
            F32x4Max { snake: f32x4_max, feature = "simd" }
            F32x4PMin { snake: f32x4_pmin, feature = "simd" }
            F32x4PMax { snake: f32x4_pmax, feature = "simd" }
            F64x2Ceil { snake: f64x2_ceil, feature = "simd" }
            F64x2Floor { snake: f64x2_floor, feature = "simd" }
            F64x2Trunc { snake: f64x2_trunc, feature = "simd" }
            F64x2Nearest { snake: f64x2_nearest, feature = "simd" }
            F64x2Abs { snake: f64x2_abs, feature = "simd" }
            F64x2Neg { snake: f64x2_neg, feature = "simd" }
            F64x2Sqrt { snake: f64x2_sqrt, feature = "simd" }
            F64x2Add { snake: f64x2_add, feature = "simd" }
            F64x2Sub { snake: f64x2_sub, feature = "simd" }
            F64x2Mul { snake: f64x2_mul, feature = "simd" }
            F64x2Div { snake: f64x2_div, feature = "simd" }
            F64x2Min { snake: f64x2_min, feature = "simd" }
            F64x2Max { snake: f64x2_max, feature = "simd" }
            F64x2PMin { snake: f64x2_pmin, feature = "simd" }
            F64x2PMax { snake: f64x2_pmax, feature = "simd" }
            I32x4TruncSatF32x4S { snake: i32x4_trunc_sat_f32x4_s, feature = "simd" }
            I32x4TruncSatF32x4U { snake: i32x4_trunc_sat_f32x4_u, feature = "simd" }
            F32x4ConvertI32x4S { snake: f32x4_convert_i32x4_s, feature = "simd" }
            F32x4ConvertI32x4U { snake: f32x4_convert_i32x4_u, feature = "simd" }
            I32x4TruncSatF64x2SZero { snake: i32x4_trunc_sat_f64x2_szero, feature = "simd" }
            I32x4TruncSatF64x2UZero { snake: i32x4_trunc_sat_f64x2_uzero, feature = "simd" }
            F64x2ConvertLowI32x4S { snake: f64x2_convert_low_i32x4_s, feature = "simd" }
            F64x2ConvertLowI32x4U { snake: f64x2_convert_low_i32x4_u, feature = "simd" }
            F32x4DemoteF64x2Zero { snake: f32x4_demote_f64x2_zero, feature = "simd" }
            F64x2PromoteLowF32x4 { snake: f64x2_promote_low_f32x4, feature = "simd" }

            // relaxed-simd
            I8x16RelaxedSwizzle { snake: i8x16_relaxed_swizzle, feature = "simd" }
            I32x4RelaxedTruncF32x4S { snake: i32x4_relaxed_trunc_f32x4_s, feature = "simd" }
            I32x4RelaxedTruncF32x4U { snake: i32x4_relaxed_trunc_f32x4_u, feature = "simd" }
            I32x4RelaxedTruncF64x2SZero { snake: i32x4_relaxed_trunc_f64x2_szero, feature = "simd" }
            I32x4RelaxedTruncF64x2UZero { snake: i32x4_relaxed_trunc_f64x2_uzero, feature = "simd" }
            F32x4RelaxedMadd { snake: f32x4_relaxed_madd, feature = "simd" }
            F32x4RelaxedNmadd { snake: f32x4_relaxed_nmadd, feature = "simd" }
            F64x2RelaxedMadd { snake: f64x2_relaxed_madd, feature = "simd" }
            F64x2RelaxedNmadd { snake: f64x2_relaxed_nmadd, feature = "simd" }
            I8x16RelaxedLaneselect { snake: i8x16_relaxed_laneselect, feature = "simd" }
            I16x8RelaxedLaneselect { snake: i16x8_relaxed_laneselect, feature = "simd" }
            I32x4RelaxedLaneselect { snake: i32x4_relaxed_laneselect, feature = "simd" }
            I64x2RelaxedLaneselect { snake: i64x2_relaxed_laneselect, feature = "simd" }
            F32x4RelaxedMin { snake: f32x4_relaxed_min, feature = "simd" }
            F32x4RelaxedMax { snake: f32x4_relaxed_max, feature = "simd" }
            F64x2RelaxedMin { snake: f64x2_relaxed_min, feature = "simd" }
            F64x2RelaxedMax { snake: f64x2_relaxed_max, feature = "simd" }
            I16x8RelaxedQ15mulrS { snake: i16x8_relaxed_q15mulr_s, feature = "simd" }
            I16x8RelaxedDotI8x16I7x16S { snake: i16x8_relaxed_dot_i8x16_i7x16_s, feature = "simd" }
            I32x4RelaxedDotI8x16I7x16AddS { snake: i32x4_relaxed_dot_i8x16_i7x16_add_s, feature = "simd" }
        }
    };
}

macro_rules! define_wasm_operator {
    ( $($camel:ident { snake: $snake:ident $(, feature = $feature:literal )? } )* ) => {
        /// A Wasm operator supported by Wasmi.
        #[derive(Debug, Copy, Clone)]
        #[non_exhaustive]
        pub enum WasmOperator {
            $(
                $( #[cfg(feature = $feature)] )?
                $camel
            ),*
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
    ( $($camel:ident { snake: $snake:ident $(, feature = $feature:literal )? } )* ) => {
        /// The fuel cost of each operator in a table.
        #[derive(Debug, Copy, Clone)]
        pub struct OperatorCost {
            $(
                $( #[cfg(feature = $feature)] )?
                pub $snake: u8
            ),*
        }

        impl Default for OperatorCost {
            fn default() -> Self {
                Self {
                    $(
                        $( #[cfg(feature = $feature)] )?
                        $snake: default_cost!($camel)
                    ),*
                }
            }
        }

        impl OperatorCost {
            /// Returns the cost for `op`.
            #[inline]
            pub fn cost(&self, op: WasmOperator) -> u64 {
                let cost = match op {
                    $(
                        $( #[cfg(feature = $feature)] )?
                        WasmOperator::$camel => self.$snake,
                    )*
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
    ( $($camel:ident { snake: $snake:ident $(, feature = $feature:literal )? } )* ) => {
        impl OperatorCostStrategy {
            /// Returns Wasmi's default costs for `op`.
            fn default_cost(op: WasmOperator) -> u64 {
                match op {
                    $(
                        $( #[cfg(feature = $feature)] )?
                        WasmOperator::$camel => default_cost!($camel)
                    ),*
                }
            }
        }
    };
}
for_each_wasm_operator!(impl_operator_cost_strategy);
