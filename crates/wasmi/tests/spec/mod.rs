mod context;
mod descriptor;
mod error;
mod profile;
mod run;

use self::{
    context::TestContext,
    descriptor::{TestDescriptor, TestSpan},
    error::TestError,
    profile::TestProfile,
};
use wasmi::Config;

/// Creates the proper [`Config`] for testing.
fn mvp_config() -> Config {
    let mut config = Config::default();
    config
        .wasm_mutable_global(false)
        .wasm_saturating_float_to_int(false)
        .wasm_sign_extension(false)
        .wasm_multi_value(false);
    config
}

/// Run Wasm spec test suite using MVP `wasmi` configuration.
///
/// # Note
///
/// The Wasm MVP has no Wasm proposals enabled.
fn run_wasm_spec_test(file_name: &str) {
    let mut config = mvp_config();
    config.wasm_mutable_global(true);
    self::run::run_wasm_spec_test(file_name, config)
}

macro_rules! define_local_tests {
    ( $( $(#[$attr:meta])* fn $test_name:ident($file_name:expr); )* ) => {
        $(
            #[test]
            $( #[$attr] )*
            fn $test_name() {
                run_wasm_spec_test(&format!("local/{}", $file_name))
            }
        )*
    };
}

mod missing_features {
    use super::mvp_config;

    /// Run Wasm spec test suite using `multi-value` Wasm proposal enabled.
    fn run_wasm_spec_test(file_name: &str) {
        super::run::run_wasm_spec_test(file_name, mvp_config())
    }

    define_local_tests! {
        fn wasm_mutable_global("missing-features/mutable-global-disabled");
        fn wasm_sign_extension("missing-features/sign-extension-disabled");
        fn wasm_saturating_float_to_int("missing-features/saturating-float-to-int-disabled");
    }
}

macro_rules! define_spec_tests {
    ( $( $(#[$attr:meta])* fn $test_name:ident($file_name:expr); )* ) => {
        $(
            #[test]
            $( #[$attr] )*
            fn $test_name() {
                run_wasm_spec_test(&format!("testsuite/{}", $file_name))
            }
        )*
    };
}

mod saturating_float_to_int {
    use super::mvp_config;

    /// Run Wasm spec test suite using `multi-value` Wasm proposal enabled.
    fn run_wasm_spec_test(file_name: &str) {
        let mut config = mvp_config();
        config.wasm_saturating_float_to_int(true);
        super::run::run_wasm_spec_test(file_name, config)
    }

    define_spec_tests! {
        fn wasm_conversions("proposals/nontrapping-float-to-int-conversions/conversions");
    }
}

mod sign_extension_ops {
    use super::mvp_config;

    /// Run Wasm spec test suite using `multi-value` Wasm proposal enabled.
    fn run_wasm_spec_test(file_name: &str) {
        let mut config = mvp_config();
        config.wasm_sign_extension(true);
        super::run::run_wasm_spec_test(file_name, config)
    }

    define_spec_tests! {
        fn wasm_i32("proposals/sign-extension-ops/i32");
        fn wasm_i64("proposals/sign-extension-ops/i64");
    }
}

mod multi_value {
    use super::mvp_config;

    /// Run Wasm spec test suite using `multi-value` Wasm proposal enabled.
    fn run_wasm_spec_test(file_name: &str) {
        let mut config = mvp_config();
        config.wasm_multi_value(true);
        super::run::run_wasm_spec_test(file_name, config)
    }

    define_spec_tests! {
        fn wasm_binary("proposals/multi-value/binary");
        fn wasm_block("proposals/multi-value/block");
        fn wasm_br("proposals/multi-value/br");
        fn wasm_call("proposals/multi-value/call");
        fn wasm_call_indirect("proposals/multi-value/call_indirect");
        fn wasm_fac("proposals/multi-value/fac");
        fn wasm_func("proposals/multi-value/func");
        fn wasm_if("proposals/multi-value/if");
        fn wasm_loop("proposals/multi-value/loop");
        fn wasm_type("proposals/multi-value/type");
    }
}

mod mutable_global {
    use super::mvp_config;

    /// Run Wasm spec test suite using `multi-value` Wasm proposal enabled.
    fn run_wasm_spec_test(file_name: &str) {
        let mut config = mvp_config();
        config.wasm_mutable_global(true);
        super::run::run_wasm_spec_test(file_name, config)
    }

    define_spec_tests! {
        fn wasm_globals("proposals/mutable-global/globals");
        // We expect failure for this test case temporarily because `wasmi` already implements
        // the intended behavior using the semantics introduced in the `bulk-memory`
        // Wasm proposal withot having `bulk-memory` Wasm proposal support:
        // https://github.com/WebAssembly/bulk-memory-operations/blob/master/proposals/bulk-memory-operations/Overview.md#segment-initialization
        //
        // Unignore the test case and remove this comment once `bulk-memory` has been implemented.
        #[should_panic] fn wasm_linking("proposals/mutable-global/linking");
    }
}

mod bulk_memory {
    use super::mvp_config;

    /// Run Wasm spec test suite using `multi-value` Wasm proposal enabled.
    fn run_wasm_spec_test(file_name: &str) {
        let mut config = mvp_config();
        // For some reason we need to enable `mutable-global` Wasm proposal
        // to properly pass all the `bulk-memory` Wasm spec tests.
        config.wasm_mutable_global(true);
        config.wasm_bulk_memory(true);
        super::run::run_wasm_spec_test(file_name, config)
    }

    define_spec_tests! {
        fn wasm_binary("proposals/bulk-memory-operations/binary");
        fn wasm_bulk("proposals/bulk-memory-operations/bulk");
        fn wasm_custom("proposals/bulk-memory-operations/custom");
        fn wasm_data("proposals/bulk-memory-operations/data");
        fn wasm_elem("proposals/bulk-memory-operations/elem");
        fn wasm_imports("proposals/bulk-memory-operations/imports");
        // Even though `wasmi` does not implement the `bulk-memory` Wasm proposal
        // at the time of this writing we run this specific test from its test suite
        // because `wasmi` already uses the changed semantics for segment instantiation
        // coming from the `bulk-memory` Wasm proposal.
        // This is why the other `linking` tests are expected to fail.
        fn wasm_linking("proposals/bulk-memory-operations/linking");
        fn wasm_memory_copy("proposals/bulk-memory-operations/memory_copy");
        fn wasm_memory_fill("proposals/bulk-memory-operations/memory_fill");
        fn wasm_memory_init("proposals/bulk-memory-operations/memory_init");
        fn wasm_table_copy("proposals/bulk-memory-operations/table_copy");
        fn wasm_table_init("proposals/bulk-memory-operations/table_init");
    }
}

define_spec_tests! {
    fn wasm_address("address");
    fn wasm_align("align");
    fn wasm_binary("binary");
    fn wasm_binary_leb128("binary-leb128");
    fn wasm_block("block");
    fn wasm_br("br");
    fn wasm_br_if("br_if");
    fn wasm_br_table("br_table");
    fn wasm_break_drop("break-drop");
    fn wasm_call("call");
    fn wasm_call_indirect("call_indirect");
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
    fn wasm_globals("globals");
    fn wasm_i32("i32");
    fn wasm_i64("i64");
    fn wasm_if("if");
    fn wasm_imports("imports");
    fn inline_module("inline-module");
    fn wasm_int_exprs("int_exprs");
    fn wasm_int_literals("int_literals");
    fn wasm_labels("labels");
    fn wasm_left_to_right("left-to-right");
    // We expect failure for this test case temporarily because `wasmi` already implements
    // the intended behavior using the semantics introduced in the `bulk-memory`
    // Wasm proposal withot having `bulk-memory` Wasm proposal support:
    // https://github.com/WebAssembly/bulk-memory-operations/blob/master/proposals/bulk-memory-operations/Overview.md#segment-initialization
    //
    // Unignore the test case and remove this comment once `bulk-memory` has been implemented.
    #[should_panic] fn wasm_linking("linking");
    fn wasm_loop("loop");
    fn wasm_load("load");
    fn wasm_local_get("local_get");
    fn wasm_local_set("local_set");
    fn wasm_local_tee("local_tee");
    fn wasm_memory("memory");
    fn wasm_memory_redundancy("memory_redundancy");
    fn wasm_memory_trap("memory_trap");
    fn wasm_memory_grow("memory_grow");
    fn wasm_memory_size("memory_size");
    fn wasm_names("names");
    fn wasm_nop("nop");
    fn wasm_return("return");
    fn wasm_select("select");
    fn wasm_skip_stack_guard_page("skip-stack-guard-page");
    fn wasm_stack("stack");
    fn wasm_start("start");
    fn wasm_store("store");
    fn wasm_switch("switch");
    fn wasm_table("table");
    fn wasm_token("token");
    fn wasm_traps("traps");
    fn wasm_type("type");
    fn wasm_typecheck("typecheck");
    fn wasm_unreachable("unreachable");
    // I do not remember why we ignored this test but it might either be
    // due to some updated semantics in Wasm proposals that we already
    // implement or due to actual bugs in `wasmi`. Need to investigate.
    #[ignore] fn wasm_unreached_invalid("unreached-invalid");
    fn wasm_unwind("unwind");
    fn wasm_utf8_custom_section_id("utf8-custom-section-id");
    fn wasm_utf8_import_field("utf8-import-field");
    fn wasm_utf8_import_module("utf8-import-module");
    fn wasm_utf8_invalid_encoding("utf8-invalid-encoding");
}
