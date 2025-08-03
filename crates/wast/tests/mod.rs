use wasmi::{CompilationMode, Config};
use wasmi_wast::{ParsingMode, RunnerConfig, WastRunner};

/// Runs the Wasm test spec identified by the given name.
fn process_wast(path: &'static str, wast: &'static str, config: RunnerConfig) {
    let mut runner = WastRunner::new(config);
    if let Err(error) = runner.register_spectest() {
        panic!("{path}: failed to setup Wasm spectest module: {error}");
    }
    if let Err(error) = runner.process_directives(path, wast) {
        panic!("{error:#}")
    }
}

macro_rules! define_test {
    (
        let config = $get_config:expr;

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
        .wasm_mutable_global(false)
        .wasm_saturating_float_to_int(false)
        .wasm_sign_extension(false)
        .wasm_multi_value(false)
        .wasm_multi_memory(false)
        .wasm_simd(false)
        .wasm_memory64(false);
    config
}

/// Create a [`Config`] with all Wasm feature supported by Wasmi enabled.
///
/// # Note
///
/// The Wasm MVP has no Wasm proposals enabled.
fn test_config(consume_fuel: bool, parsing_mode: ParsingMode) -> RunnerConfig {
    let mut config = mvp_config();
    // We have to enable the `mutable-global` Wasm proposal because
    // it seems that the entire Wasm spec test suite is already built
    // on the basis of its semantics.
    config
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
        .wasm_simd(true)
        .consume_fuel(consume_fuel)
        .compilation_mode(CompilationMode::Eager);
    RunnerConfig {
        config,
        parsing_mode,
    }
}

macro_rules! foreach_test {
    ( $mac:ident, $( $args:tt )* ) => {
        $mac! {
            $( $args )*

            fn spec_address("address");
            fn spec_align("align");
            fn spec_binary_leb128("binary-leb128");
            fn spec_binary("binary");
            fn spec_block("block");
            fn spec_br("br");
            fn spec_br_if("br_if");
            fn spec_br_table("br_table");
            fn spec_bulk("bulk");
            fn spec_call("call");
            fn spec_call_indirect("call_indirect");
            fn spec_extended_const_data("proposals/extended-const/data");
            fn spec_extended_const_elem("proposals/extended-const/elem");
            fn spec_extended_const_global("proposals/extended-const/global");
            fn spec_return_call("proposals/tail-call/return_call");
            fn spec_return_call_indirect("proposals/tail-call/return_call_indirect");
            fn spec_comments("comments");
            fn spec_const("const");
            fn spec_conversions("conversions");
            fn spec_custom("custom");
            fn spec_data("data");
            fn spec_elem("elem");
            fn spec_endianness("endianness");
            fn spec_exports("exports");
            fn spec_f32("f32");
            fn spec_f32_bitwise("f32_bitwise");
            fn spec_f32_cmp("f32_cmp");
            fn spec_f64("f64");
            fn spec_f64_bitwise("f64_bitwise");
            fn spec_f64_cmp("f64_cmp");
            fn spec_fac("fac");
            fn spec_float_exprs("float_exprs");
            fn spec_float_literals("float_literals");
            fn spec_float_memory("float_memory");
            fn spec_float_misc("float_misc");
            fn spec_forward("forward");
            fn spec_func("func");
            fn spec_func_ptrs("func_ptrs");
            fn spec_global("global");
            fn spec_i32("i32");
            fn spec_i64("i64");
            fn spec_if("if");
            fn spec_imports("imports");
            fn spec_inline_module("inline-module");
            fn spec_int_exprs("int_exprs");
            fn spec_int_literals("int_literals");
            fn spec_labels("labels");
            fn spec_left_to_right("left-to-right");
            fn spec_linking("linking");
            fn spec_load("load");
            fn spec_local_get("local_get");
            fn spec_local_set("local_set");
            fn spec_local_tee("local_tee");
            fn spec_loop("loop");
            fn spec_memory("memory");
            fn spec_memory_copy("memory_copy");
            fn spec_memory_fill("memory_fill");
            fn spec_memory_grow("memory_grow");
            fn spec_memory_init("memory_init");
            fn spec_memory_redundancy("memory_redundancy");
            fn spec_memory_size("memory_size");
            fn spec_memory_trap("memory_trap");
            fn spec_obsolete_keywords("obsolete-keywords");
            fn spec_names("names");
            fn spec_nop("nop");
            fn spec_ref_func("ref_func");
            fn spec_ref_is_null("ref_is_null");
            fn spec_ref_null("ref_null");
            fn spec_return("return");
            fn spec_select("select");
            fn spec_skip_stack_guard_page("skip-stack-guard-page");
            fn spec_stack("stack");
            fn spec_start("start");
            fn spec_store("store");
            fn spec_switch("switch");
            fn spec_table_sub("table-sub");
            fn spec_table("table");
            fn spec_table_copy("table_copy");
            fn spec_table_fill("table_fill");
            fn spec_table_get("table_get");
            fn spec_table_grow("table_grow");
            fn spec_table_init("table_init");
            fn spec_table_set("table_set");
            fn spec_table_size("table_size");
            fn spec_token("token");
            fn spec_traps("traps");
            fn spec_type("type");
            fn spec_unreachable("unreachable");
            fn spec_unreached_invalid("unreached-invalid");
            fn spec_unreached_valid("unreached-valid");
            fn spec_unwind("unwind");
            fn spec_utf8_custom_section_id("utf8-custom-section-id");
            fn spec_utf8_import_field("utf8-import-field");
            fn spec_utf8_import_module("utf8-import-module");
            fn spec_utf8_invalid_encoding("utf8-invalid-encoding");
            fn spec_wide_arithmetic("proposals/wide-arithmetic/wide-arithmetic");

            // Wasmi specific test cases and regression tests.
            fn wasmi_wide_arithmetic("../wasmi/tests/wide-arithmetic");
            fn wasmi_replace_result("../wasmi/tests/replace-result");
            fn wasmi_local_tee("../wasmi/tests/local-tee");
            fn wasmi_if("../wasmi/tests/if");
            fn wasmi_fuse_cmp("../wasmi/tests/fuse-cmp");
            fn wasmi_select("../wasmi/tests/select");
            fn wasmi_preserve_locals("../wasmi/tests/preserve-locals");
            fn wasmi_many_inout("../wasmi/tests/many-inout");
            fn wasmi_copy_span("../wasmi/tests/copy-span");
            fn wasmi_audit("../wasmi/tests/audit");
            fn wasmi_i32_add("../wasmi/tests/op/i32-add");
            fn wasmi_i32_mul("../wasmi/tests/op/i32-mul");
            fn wasmi_i64_add("../wasmi/tests/op/i64-add");
            fn wasmi_i64_mul("../wasmi/tests/op/i64-mul");

            // Wasm `simd` tests
            fn spec_simd_address("simd_address");
            fn spec_simd_align("simd_align");
            fn spec_simd_bit_shift("simd_bit_shift");
            fn spec_simd_bitwise("simd_bitwise");
            fn spec_simd_boolean("simd_boolean");
            fn spec_simd_const("simd_const");
            fn spec_simd_conversions("simd_conversions");
            fn spec_simd_f32x4("simd_f32x4");
            fn spec_simd_f32x4_arith("simd_f32x4_arith");
            fn spec_simd_f32x4_cmp("simd_f32x4_cmp");
            fn spec_simd_f32x4_pmin_pmax("simd_f32x4_pmin_pmax");
            fn spec_simd_f32x4_rounding("simd_f32x4_rounding");
            fn spec_simd_f64x2("simd_f64x2");
            fn spec_simd_f64x2_arith("simd_f64x2_arith");
            fn spec_simd_f64x2_cmp("simd_f64x2_cmp");
            fn spec_simd_f64x2_pmin_pmax("simd_f64x2_pmin_pmax");
            fn spec_simd_f64x2_rounding("simd_f64x2_rounding");
            fn spec_simd_i16x8_arith("simd_i16x8_arith");
            fn spec_simd_i16x8_arith2("simd_i16x8_arith2");
            fn spec_simd_i16x8_cmp("simd_i16x8_cmp");
            fn spec_simd_i16x8_extadd_pairwise_i8x16("simd_i16x8_extadd_pairwise_i8x16");
            fn spec_simd_i16x8_extmul_i8x16("simd_i16x8_extmul_i8x16");
            fn spec_simd_i16x8_q15mulr_sat_s("simd_i16x8_q15mulr_sat_s");
            fn spec_simd_i16x8_sat_arith("simd_i16x8_sat_arith");
            fn spec_simd_i32x4_arith("simd_i32x4_arith");
            fn spec_simd_i32x4_arith2("simd_i32x4_arith2");
            fn spec_simd_i32x4_cmp("simd_i32x4_cmp");
            fn spec_simd_i32x4_dot_i16x8("simd_i32x4_dot_i16x8");
            fn spec_simd_i32x4_extadd_pairwise_i16x8("simd_i32x4_extadd_pairwise_i16x8");
            fn spec_simd_i32x4_extmul_i16x8("simd_i32x4_extmul_i16x8");
            fn spec_simd_i32x4_trunc_sat_f32x4("simd_i32x4_trunc_sat_f32x4");
            fn spec_simd_i32x4_trunc_sat_f64x2("simd_i32x4_trunc_sat_f64x2");
            fn spec_simd_i64x2_arith("simd_i64x2_arith");
            fn spec_simd_i64x2_arith2("simd_i64x2_arith2");
            fn spec_simd_i64x2_cmp("simd_i64x2_cmp");
            fn spec_simd_i64x2_extmul_i32x4("simd_i64x2_extmul_i32x4");
            fn spec_simd_i8x16_arith("simd_i8x16_arith");
            fn spec_simd_i8x16_arith2("simd_i8x16_arith2");
            fn spec_simd_i8x16_cmp("simd_i8x16_cmp");
            fn spec_simd_i8x16_sat_arith("simd_i8x16_sat_arith");
            fn spec_simd_int_to_int_extend("simd_int_to_int_extend");
            fn spec_simd_lane("simd_lane");
            fn spec_simd_linking("simd_linking");
            fn spec_simd_load("simd_load");
            fn spec_simd_load16_lane("simd_load16_lane");
            fn spec_simd_load32_lane("simd_load32_lane");
            fn spec_simd_load64_lane("simd_load64_lane");
            fn spec_simd_load8_lane("simd_load8_lane");
            fn spec_simd_load_splat("simd_load_splat");
            fn spec_simd_load_extend("simd_load_extend");
            fn spec_simd_load_zero("simd_load_zero");
            fn spec_simd_splat("simd_splat");
            fn spec_simd_store("simd_store");
            fn spec_simd_store16_lane("simd_store16_lane");
            fn spec_simd_store32_lane("simd_store32_lane");
            fn spec_simd_store64_lane("simd_store64_lane");
            fn spec_simd_store8_lane("simd_store8_lane");

            // Wasm `relaxed-simd` tests
            fn spec_relaxed_simd_i16x8_relaxed_q15mulr_s("proposals/relaxed-simd/i16x8_relaxed_q15mulr_s");
            fn spec_relaxed_simd_i32x4_relaxed_trunc("proposals/relaxed-simd/i32x4_relaxed_trunc");
            fn spec_relaxed_simd_i8x16_relaxed_swizzle("proposals/relaxed-simd/i8x16_relaxed_swizzle");
            fn spec_relaxed_simd_relaxed_dot_product("proposals/relaxed-simd/relaxed_dot_product");
            fn spec_relaxed_simd_relaxed_laneselect("proposals/relaxed-simd/relaxed_laneselect");
            fn spec_relaxed_simd_relaxed_madd_nmadd("proposals/relaxed-simd/relaxed_madd_nmadd");
            fn spec_relaxed_simd_relaxed_min_max("proposals/relaxed-simd/relaxed_min_max");
        }
    };
}

macro_rules! foreach_test_multi_memory {
    ( $mac:ident, $( $args:tt )* ) => {
        $mac! {
            $( $args )*

            fn spec_multi_memory_align("proposals/multi-memory/align");
            fn spec_multi_memory_address0("proposals/multi-memory/address0");
            fn spec_multi_memory_address1("proposals/multi-memory/address1");
            fn spec_multi_memory_align0("proposals/multi-memory/align0");
            fn spec_multi_memory_binary("proposals/multi-memory/binary");
            fn spec_multi_memory_binary0("proposals/multi-memory/binary0");
            fn spec_multi_memory_data_drop0("proposals/multi-memory/data_drop0");
            fn spec_multi_memory_data("proposals/multi-memory/data");
            fn spec_multi_memory_data0("proposals/multi-memory/data0");
            fn spec_multi_memory_data1("proposals/multi-memory/data1");
            fn spec_multi_memory_exports0("proposals/multi-memory/exports0");
            fn spec_multi_memory_float_exprs0("proposals/multi-memory/float_exprs0");
            fn spec_multi_memory_float_exprs1("proposals/multi-memory/float_exprs1");
            fn spec_multi_memory_float_memory0("proposals/multi-memory/float_memory0");
            fn spec_multi_memory_imports("proposals/multi-memory/imports");
            fn spec_multi_memory_imports0("proposals/multi-memory/imports0");
            fn spec_multi_memory_imports1("proposals/multi-memory/imports1");
            fn spec_multi_memory_imports2("proposals/multi-memory/imports2");
            fn spec_multi_memory_imports3("proposals/multi-memory/imports3");
            fn spec_multi_memory_imports4("proposals/multi-memory/imports4");
            fn spec_multi_memory_linking0("proposals/multi-memory/linking0");
            fn spec_multi_memory_linking1("proposals/multi-memory/linking1");
            fn spec_multi_memory_linking2("proposals/multi-memory/linking2");
            fn spec_multi_memory_linking3("proposals/multi-memory/linking3");
            fn spec_multi_memory_load("proposals/multi-memory/load");
            fn spec_multi_memory_load0("proposals/multi-memory/load0");
            fn spec_multi_memory_load1("proposals/multi-memory/load1");
            fn spec_multi_memory_load2("proposals/multi-memory/load2");
            fn spec_multi_memory_memory_copy0("proposals/multi-memory/memory_copy0");
            fn spec_multi_memory_memory_copy1("proposals/multi-memory/memory_copy1");
            fn spec_multi_memory_memory_fill0("proposals/multi-memory/memory_fill0");
            fn spec_multi_memory_memory_grow("proposals/multi-memory/memory_grow");
            fn spec_multi_memory_memory_init0("proposals/multi-memory/memory_init0");
            fn spec_multi_memory_memory_size("proposals/multi-memory/memory_size");
            fn spec_multi_memory_memory_size0("proposals/multi-memory/memory_size0");
            fn spec_multi_memory_memory_size1("proposals/multi-memory/memory_size1");
            fn spec_multi_memory_memory_size2("proposals/multi-memory/memory_size2");
            fn spec_multi_memory_memory_size3("proposals/multi-memory/memory_size3");
            fn spec_multi_memory_memory_trap0("proposals/multi-memory/memory_trap0");
            fn spec_multi_memory_memory_trap1("proposals/multi-memory/memory_trap1");
            fn spec_multi_memory_memory_multi("proposals/multi-memory/memory-multi");
            fn spec_multi_memory_memory("proposals/multi-memory/memory");
            fn spec_multi_memory_simd_memory("proposals/multi-memory/simd_memory-multi");
            fn spec_multi_memory_start0("proposals/multi-memory/start0");
            fn spec_multi_memory_store("proposals/multi-memory/store");
            fn spec_multi_memory_store0("proposals/multi-memory/store0");
            fn spec_multi_memory_store1("proposals/multi-memory/store1");
            fn spec_multi_memory_traps0("proposals/multi-memory/traps0");
        }
    };
}

macro_rules! foreach_test_cps {
    ( $mac:ident, $( $args:tt )* ) => {
        $mac! {
            $( $args )*

            fn spec_custom_page_sizes("proposals/custom-page-sizes/custom-page-sizes");
            fn spec_custom_page_sizes_invalid("proposals/custom-page-sizes/custom-page-sizes-invalid");
            fn spec_custom_page_sizes_memory_max("proposals/custom-page-sizes/memory_max");
            fn spec_custom_page_sizes_memory_max64("proposals/custom-page-sizes/memory_max_i64");
        }
    };
}

macro_rules! foreach_test_memory64 {
    ( $mac:ident, $( $args:tt )* ) => {
        $mac! {
            $( $args )*

            fn spec_address64("memory64/address64");
            fn spec_align64("memory64/align64");
            fn spec_call_indirect64("memory64/call_indirect");
            fn spec_endianness64("memory64/endianness64");
            fn spec_float_memory64("memory64/float_memory64");
            fn spec_load64("memory64/load64");
            fn spec_memory_grow64("memory64/memory_grow64");
            fn spec_memory_trap64("memory64/memory_trap64");
            fn spec_memory_redundancy64("memory64/memory_redundancy64");
            fn spec_memory64("memory64/memory64");
            fn spec_memory_copy64("memory64/memory_copy");
            fn spec_memory_fill64("memory64/memory_fill");
            fn spec_memory_init64("memory64/memory_init");
            fn spec_imports64("memory64/imports");
            fn spec_table64("memory64/table");
            fn spec_table_copy_mixed("memory64/table_copy_mixed");
        }
    };
}

macro_rules! foreach_test_missing_features {
    ( $mac:ident, $( $args:tt )* ) => {
        $mac! {
            $( $args )*

            fn local_mutable_global_disabled("mutable-global-disabled");
            fn local_saturating_float_to_int_disabled("saturating-float-to-int-disabled");
            fn local_sign_extension_disabled("sign-extension-disabled");
        }
    };
}

macro_rules! include_wasm_blobs {
    (
        let folder = $test_folder:literal;

        $( $(#[$attr:meta])* fn $test_name:ident($file_name:literal); )*
    ) => {
        $(
            $( #[$attr] )*
            pub fn $test_name() -> &'static str {
                ::core::include_str!(
                    ::core::concat!($test_folder, "/", $file_name, ".wast")
                )
            }
        )*
    };
}

mod blobs {
    foreach_test! {
        include_wasm_blobs,

        let folder = "spec";
    }

    foreach_test_multi_memory! {
        include_wasm_blobs,

        let folder = "spec";
    }

    foreach_test_cps! {
        include_wasm_blobs,

        let folder = "spec";
    }

    foreach_test_memory64! {
        include_wasm_blobs,

        let folder = "wasmi";
    }

    foreach_test_missing_features! {
        include_wasm_blobs,

        let folder = "wasmi/tests/missing-features";
    }
}

mod buffered {
    use super::*;

    foreach_test! {
        define_test,

        let config = test_config(false, ParsingMode::Buffered);
    }
}

mod fueled {
    use super::*;

    foreach_test! {
        define_test,

        let config = test_config(true, ParsingMode::Buffered);
    }
}

mod streaming {
    use super::*;

    foreach_test! {
        define_test,

        let config = test_config(false, ParsingMode::Streaming);
    }
}

mod missing_features {
    use super::*;

    foreach_test_missing_features! {
        define_test,

        let config = RunnerConfig {
            config: mvp_config(),
            parsing_mode: ParsingMode::Streaming,
        };
    }
}

mod multi_memory {
    use super::*;

    fn test_config() -> RunnerConfig {
        let mut config = Config::default();
        config.wasm_memory64(false);
        let parsing_mode = ParsingMode::Buffered;
        RunnerConfig {
            config,
            parsing_mode,
        }
    }

    foreach_test_multi_memory! {
        define_test,

        let config = test_config();
    }
}

mod custom_page_sizes {
    use super::*;

    fn test_config() -> RunnerConfig {
        let mut config = Config::default();
        config.wasm_custom_page_sizes(true);
        let parsing_mode = ParsingMode::Buffered;
        RunnerConfig {
            config,
            parsing_mode,
        }
    }

    foreach_test_cps! {
        define_test,

        let config = test_config();
    }
}

mod memory64 {
    use super::*;

    fn test_config() -> RunnerConfig {
        let mut config = Config::default();
        config.wasm_memory64(true);
        let parsing_mode = ParsingMode::Buffered;
        RunnerConfig {
            config,
            parsing_mode,
        }
    }

    foreach_test_memory64! {
        define_test,

        let config = test_config();
    }
}
