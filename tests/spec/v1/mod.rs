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

macro_rules! define_tests {
    ( $( $(#[$attr:meta])* fn $test_name:ident($file_name:expr); )* ) => {
        $(
            #[test]
            $( #[$attr] )*
            fn $test_name() {
                self::run::run_wasm_spec_test($file_name)
            }
        )*
    };
}
define_tests! {
    fn wasm_address("address");
    fn wasm_align("align");
    fn wasm_binary("binary");
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
    fn wasm_custom_section("custom_section");
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
    fn wasm_get_local("get_local");
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
    #[ignore] fn wasm_linking("linking");
    fn wasm_loop("loop");
    fn wasm_memory("memory");
    fn wasm_memory_redundancy("memory_redundancy");
    fn wasm_memory_trap("memory_trap");
    fn wasm_names("names");
    fn wasm_nop("nop");
    fn wasm_resizing("resizing");
    fn wasm_return("return");
    fn wasm_select("select");
    fn wasm_set_local("set_local");
    #[ignore] fn wasm_skip_stack_guard_page("skip-stack-guard-page");
    fn wasm_stack("stack");
    fn wasm_start("start");
    fn wasm_store_retval("store_retval");
    fn wasm_switch("switch");
    fn wasm_tee_local("tee_local");
    fn wasm_token("token");
    fn wasm_traps("traps");
    fn wasm_type("type");
    fn wasm_typecheck("typecheck");
    fn wasm_unreachable("unreachable");
    fn wasm_unreached_invalid("unreached-invalid");
    fn wasm_unwind("unwind");
    fn wasm_utf8_custom_section_id("utf8-custom-section-id");
    fn wasm_utf8_import_field("utf8-import-field");
    fn wasm_utf8_import_module("utf8-import-module");
    fn wasm_utf8_invalid_encoding("utf8-invalid-encoding");
}
