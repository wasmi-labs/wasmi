use wasmi::{CompilationMode, Config};
use wasmi_wast::WastRunner;

/// Runs the Wasm test spec identified by the given name.
fn process_wast(path: &'static str, wast: &'static str, config: Config) {
    let mut runner = WastRunner::new(&config);
    if let Err(error) = runner.register_spectest() {
        panic!("{path}: failed to setup Wasm spectest module: {error}");
    }
    if let Err(error) = runner.process_directives(path, wast) {
        panic!("{error:#}")
    }
}

macro_rules! define_test {
    (
        config: $get_config:expr;

        $( $(#[$attr:meta])* fn $test_name:ident($file_name:literal); )*
    ) => {
        $(
            #[test]
            $( #[$attr] )*
            fn $test_name() {
                let name: &'static ::core::primitive::str = ::core::concat!($file_name);
                let file: &'static ::core::primitive::str = self::blobs::$test_name();
                process_wast(name, file, $get_config)
            }
        )*
    };
}

/// Create a [`Config`] for the Wasm MVP feature set.
fn mvp_config() -> Config {
    let mut config = Config::default();
    config
        .compilation_mode(CompilationMode::Eager)
        .wasm_mutable_global(false)
        .wasm_saturating_float_to_int(false)
        .wasm_sign_extension(false)
        .wasm_multi_value(false)
        .wasm_multi_memory(false)
        .wasm_simd(false)
        .wasm_memory64(false);
    config
}

/// If Wasmi's fuel metering is enabled or disabled.
pub enum FuelMetering {
    /// Fuel metering is disabled.
    Disabled,
    /// Fuel metering is enabled.
    Enabled,
}

/// Returns [`Config`] with `parsing_mode` and apply `adjust_config` to its [`Config`].
fn runner_config(adjust_config: impl Fn(&mut Config)) -> Config {
    let mut config = mvp_config();
    adjust_config(&mut config);
    config
}

/// Returns a closure that applies a [`Config`]'s Config for Wasm spec tests.
fn apply_spec_config(fuel_metering: FuelMetering) -> impl Fn(&mut Config) {
    move |config| {
        config
            .consume_fuel(matches!(fuel_metering, FuelMetering::Enabled))
            .wasm_mutable_global(true)
            .wasm_saturating_float_to_int(true)
            .wasm_sign_extension(true)
            .wasm_multi_value(true)
            .wasm_multi_memory(false)
            .wasm_bulk_memory(true)
            .wasm_reference_types(true)
            .wasm_tail_call(true)
            .wasm_extended_const(true)
            .wasm_wide_arithmetic(true)
            .wasm_simd(true);
    }
}

macro_rules! foreach_test {
    ( $mac:ident $( $args:tt )* ) => {
        $mac! {
            $( $args )*

            fn spec_address("spec/address");
            fn spec_align("spec/align");
            fn spec_binary_leb128("spec/binary-leb128");
            fn spec_binary("spec/binary");
            fn spec_block("spec/block");
            fn spec_br("spec/br");
            fn spec_br_if("spec/br_if");
            fn spec_br_table("spec/br_table");
            fn spec_bulk("spec/bulk");
            fn spec_call("spec/call");
            fn spec_call_indirect("spec/call_indirect");
            fn spec_extended_const_data("spec/proposals/extended-const/data");
            fn spec_extended_const_elem("spec/proposals/extended-const/elem");
            fn spec_extended_const_global("spec/proposals/extended-const/global");
            fn spec_return_call("spec/proposals/tail-call/return_call");
            fn spec_return_call_indirect("spec/proposals/tail-call/return_call_indirect");
            fn spec_comments("spec/comments");
            fn spec_const("spec/const");
            fn spec_conversions("spec/conversions");
            fn spec_custom("spec/custom");
            fn spec_data("spec/data");
            fn spec_elem("spec/elem");
            fn spec_endianness("spec/endianness");
            fn spec_exports("spec/exports");
            fn spec_f32("spec/f32");
            fn spec_f32_bitwise("spec/f32_bitwise");
            fn spec_f32_cmp("spec/f32_cmp");
            fn spec_f64("spec/f64");
            fn spec_f64_bitwise("spec/f64_bitwise");
            fn spec_f64_cmp("spec/f64_cmp");
            fn spec_fac("spec/fac");
            fn spec_float_exprs("spec/float_exprs");
            fn spec_float_literals("spec/float_literals");
            fn spec_float_memory("spec/float_memory");
            fn spec_float_misc("spec/float_misc");
            fn spec_forward("spec/forward");
            fn spec_func("spec/func");
            fn spec_func_ptrs("spec/func_ptrs");
            fn spec_global("spec/global");
            fn spec_i32("spec/i32");
            fn spec_i64("spec/i64");
            fn spec_if("spec/if");
            fn spec_imports("spec/imports");
            fn spec_inline_module("spec/inline-module");
            fn spec_int_exprs("spec/int_exprs");
            fn spec_int_literals("spec/int_literals");
            fn spec_labels("spec/labels");
            fn spec_left_to_right("spec/left-to-right");
            fn spec_linking("spec/linking");
            fn spec_load("spec/load");
            fn spec_local_get("spec/local_get");
            fn spec_local_set("spec/local_set");
            fn spec_local_tee("spec/local_tee");
            fn spec_loop("spec/loop");
            fn spec_memory("spec/memory");
            fn spec_memory_copy("spec/memory_copy");
            fn spec_memory_fill("spec/memory_fill");
            fn spec_memory_grow("spec/memory_grow");
            fn spec_memory_init("spec/memory_init");
            fn spec_memory_redundancy("spec/memory_redundancy");
            fn spec_memory_size("spec/memory_size");
            fn spec_memory_trap("spec/memory_trap");
            fn spec_obsolete_keywords("spec/obsolete-keywords");
            fn spec_names("spec/names");
            fn spec_nop("spec/nop");
            fn spec_ref_func("spec/ref_func");
            fn spec_ref_is_null("spec/ref_is_null");
            fn spec_ref_null("spec/ref_null");
            fn spec_return("spec/return");
            fn spec_select("spec/select");
            fn spec_skip_stack_guard_page("spec/skip-stack-guard-page");
            fn spec_stack("spec/stack");
            fn spec_start("spec/start");
            fn spec_store("spec/store");
            fn spec_switch("spec/switch");
            fn spec_table_sub("spec/table-sub");
            fn spec_table("spec/table");
            fn spec_table_copy("spec/table_copy");
            fn spec_table_fill("spec/table_fill");
            fn spec_table_get("spec/table_get");
            fn spec_table_grow("spec/table_grow");
            fn spec_table_init("spec/table_init");
            fn spec_table_set("spec/table_set");
            fn spec_table_size("spec/table_size");
            fn spec_token("spec/token");
            fn spec_traps("spec/traps");
            fn spec_type("spec/type");
            fn spec_unreachable("spec/unreachable");
            fn spec_unreached_invalid("spec/unreached-invalid");
            fn spec_unreached_valid("spec/unreached-valid");
            fn spec_unwind("spec/unwind");
            fn spec_utf8_custom_section_id("spec/utf8-custom-section-id");
            fn spec_utf8_import_field("spec/utf8-import-field");
            fn spec_utf8_import_module("spec/utf8-import-module");
            fn spec_utf8_invalid_encoding("spec/utf8-invalid-encoding");
            fn spec_wide_arithmetic("spec/proposals/wide-arithmetic/wide-arithmetic");

            // Wasmi specific test cases and regression tests.
            fn wasmi_wide_arithmetic("wasmi/tests/wide-arithmetic");
            fn wasmi_replace_result("wasmi/tests/replace-result");
            fn wasmi_local_tee("wasmi/tests/local-tee");
            fn wasmi_if("wasmi/tests/if");
            fn wasmi_fuse_cmp("wasmi/tests/fuse-cmp");
            fn wasmi_select("wasmi/tests/select");
            fn wasmi_preserve_locals("wasmi/tests/preserve-locals");
            fn wasmi_many_inout("wasmi/tests/many-inout");
            fn wasmi_copy_span("wasmi/tests/copy-span");
            fn wasmi_audit("wasmi/tests/audit");
            // Wasmi: binary operators
            fn wasmi_i32_add("wasmi/tests/op/i32-add");
            fn wasmi_i32_sub("wasmi/tests/op/i32-sub");
            fn wasmi_i32_mul("wasmi/tests/op/i32-mul");
            fn wasmi_i32_sdiv("wasmi/tests/op/i32-sdiv");
            fn wasmi_i64_add("wasmi/tests/op/i64-add");
            fn wasmi_i64_sub("wasmi/tests/op/i64-sub");
            fn wasmi_i64_mul("wasmi/tests/op/i64-mul");
            fn wasmi_i64_sdiv("wasmi/tests/op/i64-sdiv");
            // Wasmi: fuse br_if + cmp
            fn wasmi_fuse_br_i32_ge_s("wasmi/tests/fuse-br/i32_ge_s");
            fn wasmi_fuse_br_i32_ge_u("wasmi/tests/fuse-br/i32_ge_u");
            fn wasmi_fuse_br_i32_gt_s("wasmi/tests/fuse-br/i32_gt_s");
            fn wasmi_fuse_br_i32_gt_u("wasmi/tests/fuse-br/i32_gt_u");
            fn wasmi_fuse_br_i32_le_s("wasmi/tests/fuse-br/i32_le_s");
            fn wasmi_fuse_br_i32_le_u("wasmi/tests/fuse-br/i32_le_u");
            fn wasmi_fuse_br_i32_lt_s("wasmi/tests/fuse-br/i32_lt_s");
            fn wasmi_fuse_br_i32_lt_u("wasmi/tests/fuse-br/i32_lt_u");
            fn wasmi_fuse_br_i32_and("wasmi/tests/fuse-br/i32_and");
            fn wasmi_fuse_br_i32_or("wasmi/tests/fuse-br/i32_or");
            fn wasmi_fuse_br_i32_xor("wasmi/tests/fuse-br/i32_xor");
            fn wasmi_fuse_br_i64_ge_s("wasmi/tests/fuse-br/i64_ge_s");
            fn wasmi_fuse_br_i64_ge_u("wasmi/tests/fuse-br/i64_ge_u");
            fn wasmi_fuse_br_i64_gt_s("wasmi/tests/fuse-br/i64_gt_s");
            fn wasmi_fuse_br_i64_gt_u("wasmi/tests/fuse-br/i64_gt_u");
            fn wasmi_fuse_br_i64_le_s("wasmi/tests/fuse-br/i64_le_s");
            fn wasmi_fuse_br_i64_le_u("wasmi/tests/fuse-br/i64_le_u");
            fn wasmi_fuse_br_i64_lt_s("wasmi/tests/fuse-br/i64_lt_s");
            fn wasmi_fuse_br_i64_lt_u("wasmi/tests/fuse-br/i64_lt_u");
            fn wasmi_fuse_br_i64_and("wasmi/tests/fuse-br/i64_and");
            fn wasmi_fuse_br_i64_or("wasmi/tests/fuse-br/i64_or");
            fn wasmi_fuse_br_i64_xor("wasmi/tests/fuse-br/i64_xor");
            fn wasmi_fuse_br_f32_lt("wasmi/tests/fuse-br/f32_lt");
            fn wasmi_fuse_br_f32_le("wasmi/tests/fuse-br/f32_le");
            fn wasmi_fuse_br_f32_gt("wasmi/tests/fuse-br/f32_gt");
            fn wasmi_fuse_br_f32_ge("wasmi/tests/fuse-br/f32_ge");
            fn wasmi_fuse_br_f64_lt("wasmi/tests/fuse-br/f64_lt");
            fn wasmi_fuse_br_f64_le("wasmi/tests/fuse-br/f64_le");
            fn wasmi_fuse_br_f64_gt("wasmi/tests/fuse-br/f64_gt");
            fn wasmi_fuse_br_f64_ge("wasmi/tests/fuse-br/f64_ge");
            // Wasmi: fuse if + cmp
            fn wasmi_fuse_if_i32_ge_s("wasmi/tests/fuse-if/i32_ge_s");
            fn wasmi_fuse_if_i32_ge_u("wasmi/tests/fuse-if/i32_ge_u");
            fn wasmi_fuse_if_i32_gt_s("wasmi/tests/fuse-if/i32_gt_s");
            fn wasmi_fuse_if_i32_gt_u("wasmi/tests/fuse-if/i32_gt_u");
            fn wasmi_fuse_if_i32_le_s("wasmi/tests/fuse-if/i32_le_s");
            fn wasmi_fuse_if_i32_le_u("wasmi/tests/fuse-if/i32_le_u");
            fn wasmi_fuse_if_i32_lt_s("wasmi/tests/fuse-if/i32_lt_s");
            fn wasmi_fuse_if_i32_lt_u("wasmi/tests/fuse-if/i32_lt_u");
            fn wasmi_fuse_if_i32_and("wasmi/tests/fuse-if/i32_and");
            fn wasmi_fuse_if_i32_or("wasmi/tests/fuse-if/i32_or");
            fn wasmi_fuse_if_i32_xor("wasmi/tests/fuse-if/i32_xor");
            fn wasmi_fuse_if_i64_ge_s("wasmi/tests/fuse-if/i64_ge_s");
            fn wasmi_fuse_if_i64_ge_u("wasmi/tests/fuse-if/i64_ge_u");
            fn wasmi_fuse_if_i64_gt_s("wasmi/tests/fuse-if/i64_gt_s");
            fn wasmi_fuse_if_i64_gt_u("wasmi/tests/fuse-if/i64_gt_u");
            fn wasmi_fuse_if_i64_le_s("wasmi/tests/fuse-if/i64_le_s");
            fn wasmi_fuse_if_i64_le_u("wasmi/tests/fuse-if/i64_le_u");
            fn wasmi_fuse_if_i64_lt_s("wasmi/tests/fuse-if/i64_lt_s");
            fn wasmi_fuse_if_i64_lt_u("wasmi/tests/fuse-if/i64_lt_u");
            fn wasmi_fuse_if_i64_and("wasmi/tests/fuse-if/i64_and");
            fn wasmi_fuse_if_i64_or("wasmi/tests/fuse-if/i64_or");
            fn wasmi_fuse_if_i64_xor("wasmi/tests/fuse-if/i64_xor");
            fn wasmi_fuse_if_f32_lt("wasmi/tests/fuse-if/f32_lt");
            fn wasmi_fuse_if_f32_le("wasmi/tests/fuse-if/f32_le");
            fn wasmi_fuse_if_f32_gt("wasmi/tests/fuse-if/f32_gt");
            fn wasmi_fuse_if_f32_ge("wasmi/tests/fuse-if/f32_ge");
            fn wasmi_fuse_if_f64_lt("wasmi/tests/fuse-if/f64_lt");
            fn wasmi_fuse_if_f64_le("wasmi/tests/fuse-if/f64_le");
            fn wasmi_fuse_if_f64_gt("wasmi/tests/fuse-if/f64_gt");
            fn wasmi_fuse_if_f64_ge("wasmi/tests/fuse-if/f64_ge");
            // Wasmi: fuse select + i32.cmp
            fn wasmi_fuse_select_i32_lt_s("wasmi/tests/fuse-select/i32_lt_s");
            fn wasmi_fuse_select_i32_lt_u("wasmi/tests/fuse-select/i32_lt_u");
            fn wasmi_fuse_select_i32_le_s("wasmi/tests/fuse-select/i32_le_s");
            fn wasmi_fuse_select_i32_le_u("wasmi/tests/fuse-select/i32_le_u");
            fn wasmi_fuse_select_i32_gt_s("wasmi/tests/fuse-select/i32_gt_s");
            fn wasmi_fuse_select_i32_gt_u("wasmi/tests/fuse-select/i32_gt_u");
            fn wasmi_fuse_select_i32_ge_s("wasmi/tests/fuse-select/i32_ge_s");
            fn wasmi_fuse_select_i32_ge_u("wasmi/tests/fuse-select/i32_ge_u");
            fn wasmi_fuse_select_i32_and("wasmi/tests/fuse-select/i32_and");
            fn wasmi_fuse_select_i32_or("wasmi/tests/fuse-select/i32_or");
            fn wasmi_fuse_select_i32_xor("wasmi/tests/fuse-select/i32_xor");
            // Wasmi: fuse select + i64.cmp
            fn wasmi_fuse_select_i64_lt_s("wasmi/tests/fuse-select/i64_lt_s");
            fn wasmi_fuse_select_i64_lt_u("wasmi/tests/fuse-select/i64_lt_u");
            fn wasmi_fuse_select_i64_le_s("wasmi/tests/fuse-select/i64_le_s");
            fn wasmi_fuse_select_i64_le_u("wasmi/tests/fuse-select/i64_le_u");
            fn wasmi_fuse_select_i64_gt_s("wasmi/tests/fuse-select/i64_gt_s");
            fn wasmi_fuse_select_i64_gt_u("wasmi/tests/fuse-select/i64_gt_u");
            fn wasmi_fuse_select_i64_ge_s("wasmi/tests/fuse-select/i64_ge_s");
            fn wasmi_fuse_select_i64_ge_u("wasmi/tests/fuse-select/i64_ge_u");
            fn wasmi_fuse_select_i64_and("wasmi/tests/fuse-select/i64_and");
            fn wasmi_fuse_select_i64_or("wasmi/tests/fuse-select/i64_or");
            fn wasmi_fuse_select_i64_xor("wasmi/tests/fuse-select/i64_xor");
            // Wasmi: fuse select + f32.cmp
            fn wasmi_fuse_select_f32_lt("wasmi/tests/fuse-select/f32_lt");
            fn wasmi_fuse_select_f32_le("wasmi/tests/fuse-select/f32_le");
            fn wasmi_fuse_select_f32_gt("wasmi/tests/fuse-select/f32_gt");
            fn wasmi_fuse_select_f32_ge("wasmi/tests/fuse-select/f32_ge");
            // Wasmi: fuse select + f64.cmp
            fn wasmi_fuse_select_f64_lt("wasmi/tests/fuse-select/f64_lt");
            fn wasmi_fuse_select_f64_le("wasmi/tests/fuse-select/f64_le");
            fn wasmi_fuse_select_f64_gt("wasmi/tests/fuse-select/f64_gt");
            fn wasmi_fuse_select_f64_ge("wasmi/tests/fuse-select/f64_ge");

            // Wasm `simd` tests
            fn spec_simd_address("spec/simd_address");
            fn spec_simd_align("spec/simd_align");
            fn spec_simd_bit_shift("spec/simd_bit_shift");
            fn spec_simd_bitwise("spec/simd_bitwise");
            fn spec_simd_boolean("spec/simd_boolean");
            fn spec_simd_const("spec/simd_const");
            fn spec_simd_conversions("spec/simd_conversions");
            fn spec_simd_f32x4("spec/simd_f32x4");
            fn spec_simd_f32x4_arith("spec/simd_f32x4_arith");
            fn spec_simd_f32x4_cmp("spec/simd_f32x4_cmp");
            fn spec_simd_f32x4_pmin_pmax("spec/simd_f32x4_pmin_pmax");
            fn spec_simd_f32x4_rounding("spec/simd_f32x4_rounding");
            fn spec_simd_f64x2("spec/simd_f64x2");
            fn spec_simd_f64x2_arith("spec/simd_f64x2_arith");
            fn spec_simd_f64x2_cmp("spec/simd_f64x2_cmp");
            fn spec_simd_f64x2_pmin_pmax("spec/simd_f64x2_pmin_pmax");
            fn spec_simd_f64x2_rounding("spec/simd_f64x2_rounding");
            fn spec_simd_i16x8_arith("spec/simd_i16x8_arith");
            fn spec_simd_i16x8_arith2("spec/simd_i16x8_arith2");
            fn spec_simd_i16x8_cmp("spec/simd_i16x8_cmp");
            fn spec_simd_i16x8_extadd_pairwise_i8x16("spec/simd_i16x8_extadd_pairwise_i8x16");
            fn spec_simd_i16x8_extmul_i8x16("spec/simd_i16x8_extmul_i8x16");
            fn spec_simd_i16x8_q15mulr_sat_s("spec/simd_i16x8_q15mulr_sat_s");
            fn spec_simd_i16x8_sat_arith("spec/simd_i16x8_sat_arith");
            fn spec_simd_i32x4_arith("spec/simd_i32x4_arith");
            fn spec_simd_i32x4_arith2("spec/simd_i32x4_arith2");
            fn spec_simd_i32x4_cmp("spec/simd_i32x4_cmp");
            fn spec_simd_i32x4_dot_i16x8("spec/simd_i32x4_dot_i16x8");
            fn spec_simd_i32x4_extadd_pairwise_i16x8("spec/simd_i32x4_extadd_pairwise_i16x8");
            fn spec_simd_i32x4_extmul_i16x8("spec/simd_i32x4_extmul_i16x8");
            fn spec_simd_i32x4_trunc_sat_f32x4("spec/simd_i32x4_trunc_sat_f32x4");
            fn spec_simd_i32x4_trunc_sat_f64x2("spec/simd_i32x4_trunc_sat_f64x2");
            fn spec_simd_i64x2_arith("spec/simd_i64x2_arith");
            fn spec_simd_i64x2_arith2("spec/simd_i64x2_arith2");
            fn spec_simd_i64x2_cmp("spec/simd_i64x2_cmp");
            fn spec_simd_i64x2_extmul_i32x4("spec/simd_i64x2_extmul_i32x4");
            fn spec_simd_i8x16_arith("spec/simd_i8x16_arith");
            fn spec_simd_i8x16_arith2("spec/simd_i8x16_arith2");
            fn spec_simd_i8x16_cmp("spec/simd_i8x16_cmp");
            fn spec_simd_i8x16_sat_arith("spec/simd_i8x16_sat_arith");
            fn spec_simd_int_to_int_extend("spec/simd_int_to_int_extend");
            fn spec_simd_lane("spec/simd_lane");
            fn spec_simd_linking("spec/simd_linking");
            fn spec_simd_load("spec/simd_load");
            fn spec_simd_load16_lane("spec/simd_load16_lane");
            fn spec_simd_load32_lane("spec/simd_load32_lane");
            fn spec_simd_load64_lane("spec/simd_load64_lane");
            fn spec_simd_load8_lane("spec/simd_load8_lane");
            fn spec_simd_load_splat("spec/simd_load_splat");
            fn spec_simd_load_extend("spec/simd_load_extend");
            fn spec_simd_load_zero("spec/simd_load_zero");
            fn spec_simd_splat("spec/simd_splat");
            fn spec_simd_store("spec/simd_store");
            fn spec_simd_store16_lane("spec/simd_store16_lane");
            fn spec_simd_store32_lane("spec/simd_store32_lane");
            fn spec_simd_store64_lane("spec/simd_store64_lane");
            fn spec_simd_store8_lane("spec/simd_store8_lane");

            // Wasm `relaxed-simd` tests
            fn spec_relaxed_simd_i16x8_relaxed_q15mulr_s("spec/proposals/relaxed-simd/i16x8_relaxed_q15mulr_s");
            fn spec_relaxed_simd_i32x4_relaxed_trunc("spec/proposals/relaxed-simd/i32x4_relaxed_trunc");
            fn spec_relaxed_simd_i8x16_relaxed_swizzle("spec/proposals/relaxed-simd/i8x16_relaxed_swizzle");
            fn spec_relaxed_simd_relaxed_dot_product("spec/proposals/relaxed-simd/relaxed_dot_product");
            fn spec_relaxed_simd_relaxed_laneselect("spec/proposals/relaxed-simd/relaxed_laneselect");
            fn spec_relaxed_simd_relaxed_madd_nmadd("spec/proposals/relaxed-simd/relaxed_madd_nmadd");
            fn spec_relaxed_simd_relaxed_min_max("spec/proposals/relaxed-simd/relaxed_min_max");
        }
    };
}

macro_rules! foreach_test_multi_memory {
    ( $mac:ident $( $args:tt )* ) => {
        $mac! {
            $( $args )*

            fn spec_multi_memory_align("spec/proposals/multi-memory/align");
            fn spec_multi_memory_address0("spec/proposals/multi-memory/address0");
            fn spec_multi_memory_address1("spec/proposals/multi-memory/address1");
            fn spec_multi_memory_align0("spec/proposals/multi-memory/align0");
            fn spec_multi_memory_binary("spec/proposals/multi-memory/binary");
            fn spec_multi_memory_binary0("spec/proposals/multi-memory/binary0");
            fn spec_multi_memory_data_drop0("spec/proposals/multi-memory/data_drop0");
            fn spec_multi_memory_data("spec/proposals/multi-memory/data");
            fn spec_multi_memory_data0("spec/proposals/multi-memory/data0");
            fn spec_multi_memory_data1("spec/proposals/multi-memory/data1");
            fn spec_multi_memory_exports0("spec/proposals/multi-memory/exports0");
            fn spec_multi_memory_float_exprs0("spec/proposals/multi-memory/float_exprs0");
            fn spec_multi_memory_float_exprs1("spec/proposals/multi-memory/float_exprs1");
            fn spec_multi_memory_float_memory0("spec/proposals/multi-memory/float_memory0");
            fn spec_multi_memory_imports("spec/proposals/multi-memory/imports");
            fn spec_multi_memory_imports0("spec/proposals/multi-memory/imports0");
            fn spec_multi_memory_imports1("spec/proposals/multi-memory/imports1");
            fn spec_multi_memory_imports2("spec/proposals/multi-memory/imports2");
            fn spec_multi_memory_imports3("spec/proposals/multi-memory/imports3");
            fn spec_multi_memory_imports4("spec/proposals/multi-memory/imports4");
            fn spec_multi_memory_linking0("spec/proposals/multi-memory/linking0");
            fn spec_multi_memory_linking1("spec/proposals/multi-memory/linking1");
            fn spec_multi_memory_linking2("spec/proposals/multi-memory/linking2");
            fn spec_multi_memory_linking3("spec/proposals/multi-memory/linking3");
            fn spec_multi_memory_load("spec/proposals/multi-memory/load");
            fn spec_multi_memory_load0("spec/proposals/multi-memory/load0");
            fn spec_multi_memory_load1("spec/proposals/multi-memory/load1");
            fn spec_multi_memory_load2("spec/proposals/multi-memory/load2");
            fn spec_multi_memory_memory_copy0("spec/proposals/multi-memory/memory_copy0");
            fn spec_multi_memory_memory_copy1("spec/proposals/multi-memory/memory_copy1");
            fn spec_multi_memory_memory_fill0("spec/proposals/multi-memory/memory_fill0");
            fn spec_multi_memory_memory_grow("spec/proposals/multi-memory/memory_grow");
            fn spec_multi_memory_memory_init0("spec/proposals/multi-memory/memory_init0");
            fn spec_multi_memory_memory_size("spec/proposals/multi-memory/memory_size");
            fn spec_multi_memory_memory_size0("spec/proposals/multi-memory/memory_size0");
            fn spec_multi_memory_memory_size1("spec/proposals/multi-memory/memory_size1");
            fn spec_multi_memory_memory_size2("spec/proposals/multi-memory/memory_size2");
            fn spec_multi_memory_memory_size3("spec/proposals/multi-memory/memory_size3");
            fn spec_multi_memory_memory_trap0("spec/proposals/multi-memory/memory_trap0");
            fn spec_multi_memory_memory_trap1("spec/proposals/multi-memory/memory_trap1");
            fn spec_multi_memory_memory_multi("spec/proposals/multi-memory/memory-multi");
            fn spec_multi_memory_memory("spec/proposals/multi-memory/memory");
            fn spec_multi_memory_simd_memory("spec/proposals/multi-memory/simd_memory-multi");
            fn spec_multi_memory_start0("spec/proposals/multi-memory/start0");
            fn spec_multi_memory_store("spec/proposals/multi-memory/store");
            fn spec_multi_memory_store0("spec/proposals/multi-memory/store0");
            fn spec_multi_memory_store1("spec/proposals/multi-memory/store1");
            fn spec_multi_memory_traps0("spec/proposals/multi-memory/traps0");
        }
    };
}

macro_rules! foreach_test_cps {
    ( $mac:ident $( $args:tt )* ) => {
        $mac! {
            $( $args )*

            fn spec_custom_page_sizes("spec/proposals/custom-page-sizes/custom-page-sizes");
            fn spec_custom_page_sizes_invalid("spec/proposals/custom-page-sizes/custom-page-sizes-invalid");
            fn spec_custom_page_sizes_memory_max("spec/proposals/custom-page-sizes/memory_max");
            fn spec_custom_page_sizes_memory_max64("spec/proposals/custom-page-sizes/memory_max_i64");
        }
    };
}

macro_rules! foreach_test_memory64 {
    ( $mac:ident $( $args:tt )* ) => {
        $mac! {
            $( $args )*

            fn spec_address64("wasmi/memory64/address64");
            fn spec_align64("wasmi/memory64/align64");
            fn spec_call_indirect64("wasmi/memory64/call_indirect");
            fn spec_endianness64("wasmi/memory64/endianness64");
            fn spec_float_memory64("wasmi/memory64/float_memory64");
            fn spec_load64("wasmi/memory64/load64");
            fn spec_memory_grow64("wasmi/memory64/memory_grow64");
            fn spec_memory_trap64("wasmi/memory64/memory_trap64");
            fn spec_memory_redundancy64("wasmi/memory64/memory_redundancy64");
            fn spec_memory64("wasmi/memory64/memory64");
            fn spec_memory_copy64("wasmi/memory64/memory_copy");
            fn spec_memory_fill64("wasmi/memory64/memory_fill");
            fn spec_memory_init64("wasmi/memory64/memory_init");
            fn spec_imports64("wasmi/memory64/imports");
            fn spec_table64("wasmi/memory64/table");
            fn spec_table_copy_mixed("wasmi/memory64/table_copy_mixed");
        }
    };
}

macro_rules! foreach_test_missing_features {
    ( $mac:ident $( $args:tt )* ) => {
        $mac! {
            $( $args )*

            fn local_mutable_global_disabled("wasmi/tests/missing-features/mutable-global-disabled");
            fn local_saturating_float_to_int_disabled("wasmi/tests/missing-features/saturating-float-to-int-disabled");
            fn local_sign_extension_disabled("wasmi/tests/missing-features/sign-extension-disabled");
        }
    };
}

// Note: we include the Wasm blobs into the test binary instead of reading them
//       from file so that we can run the resulting binary with the `miri` interpreter.
macro_rules! include_wasm_blobs {
    (
        $( $(#[$attr:meta])* fn $test_name:ident($file_name:literal); )*
    ) => {
        $(
            $( #[$attr] )*
            pub fn $test_name() -> &'static str {
                ::core::include_str!(
                    ::core::concat!($file_name, ".wast")
                )
            }
        )*
    };
}

mod blobs {
    foreach_test!(include_wasm_blobs);
    foreach_test_multi_memory!(include_wasm_blobs);
    foreach_test_cps!(include_wasm_blobs);
    foreach_test_memory64!(include_wasm_blobs);
    foreach_test_missing_features!(include_wasm_blobs);
}

mod buffered {
    use super::*;

    foreach_test! {
        define_test
        config: runner_config(apply_spec_config(FuelMetering::Disabled));
    }
}

mod fueled {
    use super::*;

    foreach_test! {
        define_test
        config: runner_config(apply_spec_config(FuelMetering::Enabled));
    }
}

mod missing_features {
    use super::*;

    foreach_test_missing_features! {
        define_test
        config: runner_config(|_|());
    }
}

mod multi_memory {
    use super::*;

    foreach_test_multi_memory! {
        define_test
        config: runner_config(|config| {
            config
                .wasm_simd(true)
                .wasm_mutable_global(true)
                .wasm_multi_memory(true);
        });
    }
}

mod custom_page_sizes {
    use super::*;

    foreach_test_cps! {
        define_test
        config: runner_config(|config| {
            config
                .wasm_multi_memory(true)
                .wasm_memory64(true)
                .wasm_custom_page_sizes(true);
        });
    }
}

mod memory64 {
    use super::*;

    foreach_test_memory64! {
        define_test
        config: runner_config(|config| {
            config
                .wasm_mutable_global(true)
                .wasm_multi_value(true)
                .wasm_multi_memory(true)
                .wasm_memory64(true);
        });
    }
}
