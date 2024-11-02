mod runner;

use self::runner::{ParsingMode, RunnerConfig, WastRunner};
use wasmi::Config;

/// Runs the Wasm test spec identified by the given name.
fn process_wast(path: &'static str, wast: &'static str, config: RunnerConfig) {
    let mut runner = WastRunner::new(config);
    if let Err(error) = runner.setup_wasm_spectest_module() {
        panic!("{path}: failed to setup Wasm spectest module: {error}");
    }
    if let Err(error) = runner.process_directives(path, wast) {
        panic!("{error:#}")
    }
}

macro_rules! define_tests {
    (
        let folder = $test_folder:literal;
        let config = $get_config:expr;
        let runner = $runner_fn:path;

        $( $(#[$attr:meta])* fn $test_name:ident($file_name:literal); )*
    ) => {
        $(
            #[test]
            $( #[$attr] )*
            fn $test_name() {
                let name: &'static ::core::primitive::str = ::core::concat!($test_folder, "/", $file_name);
                let file: &'static ::core::primitive::str = self::blobs::$test_name();
                $runner_fn(name, file, $get_config)
            }
        )*
    };
}

macro_rules! define_spec_tests {
    (
        let config = $get_config:expr;
        let runner = $runner_fn:path;

        $( $(#[$attr:meta])* fn $test_name:ident($file_name:literal); )*
    ) => {
        define_tests! {
            let folder = "testsuite";
            let config = $get_config;
            let runner = $runner_fn;

            $(
                $( #[$attr] )*
                fn $test_name($file_name);
            )*
        }
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
        .wasm_multi_memory(false);
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
        .consume_fuel(consume_fuel);
    RunnerConfig {
        config,
        parsing_mode,
    }
}

macro_rules! expand_tests {
    ( $mac:ident, $( $args:tt )* ) => {
        $mac! {
            $( $args )*

            fn wasm_address("address");
            fn wasm_align("align");
            fn wasm_binary_leb128("binary-leb128");
            fn wasm_binary("binary");
            fn wasm_block("block");
            fn wasm_br("br");
            fn wasm_br_if("br_if");
            fn wasm_br_table("br_table");
            fn wasm_bulk("bulk");
            fn wasm_call("call");
            fn wasm_call_indirect("call_indirect");
            fn wasm_extended_const_data("proposals/extended-const/data");
            fn wasm_extended_const_elem("proposals/extended-const/elem");
            fn wasm_extended_const_global("proposals/extended-const/global");
            fn wasm_return_call("proposals/tail-call/return_call");
            fn wasm_return_call_indirect("proposals/tail-call/return_call_indirect");
            fn wasm_comments("comments");
            fn wasm_const("const");
            fn wasm_conversions("conversions");
            fn wasm_custom("custom");
            fn wasm_data("data");
            fn wasm_elem("elem");
            fn wasm_endianness("endianness");
            fn wasm_exports("exports");
            fn wasm_f32("f32");
            fn wasm_f32_bitwise("f32_bitwise");
            fn wasm_f32_cmp("f32_cmp");
            fn wasm_f64("f64");
            fn wasm_f64_bitwise("f64_bitwise");
            fn wasm_f64_cmp("f64_cmp");
            fn wasm_fac("fac");
            fn wasm_float_exprs("float_exprs");
            fn wasm_float_literals("float_literals");
            fn wasm_float_memory("float_memory");
            fn wasm_float_misc("float_misc");
            fn wasm_forward("forward");
            fn wasm_func("func");
            fn wasm_func_ptrs("func_ptrs");
            fn wasm_global("global");
            fn wasm_i32("i32");
            fn wasm_i64("i64");
            fn wasm_if("if");
            fn wasm_imports("imports");
            fn wasm_inline_module("inline-module");
            fn wasm_int_exprs("int_exprs");
            fn wasm_int_literals("int_literals");
            fn wasm_labels("labels");
            fn wasm_left_to_right("left-to-right");
            fn wasm_linking("linking");
            fn wasm_load("load");
            fn wasm_local_get("local_get");
            fn wasm_local_set("local_set");
            fn wasm_local_tee("local_tee");
            fn wasm_loop("loop");
            fn wasm_memory("memory");
            fn wasm_memory_copy("memory_copy");
            fn wasm_memory_fill("memory_fill");
            fn wasm_memory_grow("memory_grow");
            fn wasm_memory_init("memory_init");
            fn wasm_memory_redundancy("memory_redundancy");
            fn wasm_memory_size("memory_size");
            fn wasm_memory_trap("memory_trap");
            fn wasm_names("names");
            fn wasm_nop("nop");
            fn wasm_ref_func("ref_func");
            fn wasm_ref_is_null("ref_is_null");
            fn wasm_ref_null("ref_null");
            fn wasm_return("return");
            fn wasm_select("select");
            fn wasm_skip_stack_guard_page("skip-stack-guard-page");
            fn wasm_stack("stack");
            fn wasm_start("start");
            fn wasm_store("store");
            fn wasm_switch("switch");
            fn wasm_table_sub("table-sub");
            fn wasm_table("table");
            fn wasm_table_copy("table_copy");
            fn wasm_table_fill("table_fill");
            fn wasm_table_get("table_get");
            fn wasm_table_grow("table_grow");
            fn wasm_table_init("table_init");
            fn wasm_table_set("table_set");
            fn wasm_table_size("table_size");
            fn wasm_token("token");
            fn wasm_traps("traps");
            fn wasm_type("type");
            fn wasm_unreachable("unreachable");
            fn wasm_unreached_invalid("unreached-invalid");
            fn wasm_unreached_valid("unreached-valid");
            fn wasm_unwind("unwind");
            fn wasm_utf8_custom_section_id("utf8-custom-section-id");
            fn wasm_utf8_import_field("utf8-import-field");
            fn wasm_utf8_import_module("utf8-import-module");
            fn wasm_utf8_invalid_encoding("utf8-invalid-encoding");
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

expand_tests! {
    define_spec_tests,

    let config = test_config(false, ParsingMode::Buffered);
    let runner = process_wast;
}

macro_rules! expand_tests_mm {
    ( $mac:ident, $( $args:tt )* ) => {
        $mac! {
            $( $args )*

            fn wasm_multi_memory_address0("proposals/multi-memory/address0");
            fn wasm_multi_memory_address1("proposals/multi-memory/address1");
            fn wasm_multi_memory_align0("proposals/multi-memory/align0");
            fn wasm_multi_memory_binary("proposals/multi-memory/binary");
            fn wasm_multi_memory_binary0("proposals/multi-memory/binary0");
            fn wasm_multi_memory_data_drop0("proposals/multi-memory/data_drop0");
            fn wasm_multi_memory_data("proposals/multi-memory/data");
            fn wasm_multi_memory_data0("proposals/multi-memory/data0");
            fn wasm_multi_memory_data1("proposals/multi-memory/data1");
            fn wasm_multi_memory_exports0("proposals/multi-memory/exports0");
            fn wasm_multi_memory_float_exprs0("proposals/multi-memory/float_exprs0");
            fn wasm_multi_memory_float_exprs1("proposals/multi-memory/float_exprs1");
            fn wasm_multi_memory_float_memory0("proposals/multi-memory/float_memory0");
            fn wasm_multi_memory_imports("proposals/multi-memory/imports");
            fn wasm_multi_memory_imports0("proposals/multi-memory/imports0");
            fn wasm_multi_memory_imports1("proposals/multi-memory/imports1");
            fn wasm_multi_memory_imports2("proposals/multi-memory/imports2");
            fn wasm_multi_memory_imports3("proposals/multi-memory/imports3");
            fn wasm_multi_memory_imports4("proposals/multi-memory/imports4");
            fn wasm_multi_memory_linking0("proposals/multi-memory/linking0");
            fn wasm_multi_memory_linking1("proposals/multi-memory/linking1");
            fn wasm_multi_memory_linking2("proposals/multi-memory/linking2");
            fn wasm_multi_memory_linking3("proposals/multi-memory/linking3");
            fn wasm_multi_memory_load("proposals/multi-memory/load");
            fn wasm_multi_memory_load0("proposals/multi-memory/load0");
            fn wasm_multi_memory_load1("proposals/multi-memory/load1");
            fn wasm_multi_memory_load2("proposals/multi-memory/load2");
            fn wasm_multi_memory_memory_copy0("proposals/multi-memory/memory_copy0");
            fn wasm_multi_memory_memory_copy1("proposals/multi-memory/memory_copy1");
            fn wasm_multi_memory_memory_fill0("proposals/multi-memory/memory_fill0");
            fn wasm_multi_memory_memory_grow("proposals/multi-memory/memory_grow");
            fn wasm_multi_memory_memory_init0("proposals/multi-memory/memory_init0");
            fn wasm_multi_memory_memory_size("proposals/multi-memory/memory_size");
            fn wasm_multi_memory_memory_size0("proposals/multi-memory/memory_size0");
            fn wasm_multi_memory_memory_size1("proposals/multi-memory/memory_size1");
            fn wasm_multi_memory_memory_size2("proposals/multi-memory/memory_size2");
            fn wasm_multi_memory_memory_size3("proposals/multi-memory/memory_size3");
            fn wasm_multi_memory_memory_trap0("proposals/multi-memory/memory_trap0");
            fn wasm_multi_memory_memory_trap1("proposals/multi-memory/memory_trap1");
            fn wasm_multi_memory_memory_multi("proposals/multi-memory/memory-multi");
            fn wasm_multi_memory_memory("proposals/multi-memory/memory");
            // fn wasm_multi_memory_simd_memory("proposals/multi-memory/simd_memory-multi");
            fn wasm_multi_memory_start0("proposals/multi-memory/start0");
            fn wasm_multi_memory_store("proposals/multi-memory/store");
            fn wasm_multi_memory_store0("proposals/multi-memory/store0");
            fn wasm_multi_memory_store1("proposals/multi-memory/store1");
            fn wasm_multi_memory_traps0("proposals/multi-memory/traps0");
        }
    };
}

mod blobs {
    expand_tests! {
        include_wasm_blobs,

        let folder = "testsuite";
    }

    expand_tests_mm! {
        include_wasm_blobs,

        let folder = "testsuite";
    }
}

mod multi_memory {
    use super::*;

    fn test_config() -> RunnerConfig {
        let config = Config::default();
        let parsing_mode = ParsingMode::Buffered;
        RunnerConfig {
            config,
            parsing_mode,
        }
    }

    expand_tests_mm! {
        define_spec_tests,

        let config = test_config();
        let runner = process_wast;
    }
}

mod fueled {
    use super::*;

    expand_tests! {
        define_spec_tests,

        let config = test_config(true, ParsingMode::Buffered);
        let runner = process_wast;
    }
}

mod streaming {
    use super::*;

    expand_tests! {
        define_spec_tests,

        let config = test_config(false, ParsingMode::Streaming);
        let runner = process_wast;
    }
}
