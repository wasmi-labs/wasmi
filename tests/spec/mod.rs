mod run;

macro_rules! run_test {
    ($label: expr, $test_name: ident) => (
        #[test]
        fn $test_name() {
            self::run::spec($label)
        }
    );
}

run_test!("address", wasm_address);
run_test!("align", wasm_align);
run_test!("binary", wasm_binary);
run_test!("block", wasm_block);
run_test!("br", wasm_br);
run_test!("br_if", wasm_br_if);
run_test!("br_table", wasm_br_table);
run_test!("break-drop", wasm_break_drop);
run_test!("call", wasm_call);
run_test!("call_indirect", wasm_call_indirect);
run_test!("comments", wasm_comments);
run_test!("const", wasm_const);
run_test!("conversions", wasm_conversions);
run_test!("custom_section", wasm_custom_section);
run_test!("elem", wasm_elem);
run_test!("endianness", wasm_endianness);
run_test!("exports", wasm_exports);
run_test!("f32", wasm_f32);
run_test!("f32_bitwise", wasm_f32_bitwise);
run_test!("f32_cmp", wasm_f32_cmp);
run_test!("f64", wasm_f64);
run_test!("f64_bitwise", wasm_f64_bitwise);
run_test!("f64_cmp", wasm_f64_cmp);
run_test!("fac", wasm_fac);
run_test!("float_exprs", wasm_float_exprs);
run_test!("float_literals", wasm_float_literals);
run_test!("float_memory", wasm_float_memory);
run_test!("float_misc", wasm_float_misc);
run_test!("forward", wasm_forward);
run_test!("func", wasm_func);
run_test!("func_ptrs", wasm_func_ptrs);
run_test!("get_local", wasm_get_local);
run_test!("globals", wasm_globals);
run_test!("i32", wasm_i32);
run_test!("i64", wasm_i64);
run_test!("if", wasm_if);
run_test!("imports", wasm_imports);
run_test!("inline-module", inline_module);
run_test!("int_exprs", wasm_int_exprs);
run_test!("int_literals", wasm_int_literals);
run_test!("labels", wasm_labels);
run_test!("left-to-right", wasm_left_to_right);
run_test!("linking", wasm_linking);
run_test!("loop", wasm_loop);
run_test!("memory", wasm_memory);
run_test!("memory_redundancy", wasm_memory_redundancy);
run_test!("memory_trap", wasm_memory_trap);
run_test!("names", wasm_names);
run_test!("nop", wasm_nop);
run_test!("resizing", wasm_resizing);
run_test!("return", wasm_return);
run_test!("select", wasm_select);
run_test!("set_local", wasm_set_local);
run_test!("skip-stack-guard-page", wasm_skip_stack_guard_page);
run_test!("stack", wasm_stack);
run_test!("start", wasm_start);
run_test!("store_retval", wasm_store_retval);
run_test!("switch", wasm_switch);
run_test!("tee_local", wasm_tee_local);
run_test!("token", wasm_token);
run_test!("traps", wasm_traps);
run_test!("type", wasm_type);
run_test!("typecheck", wasm_typecheck);
run_test!("unreachable", wasm_unreachable);
run_test!("unreached-invalid", wasm_unreached_invalid);
run_test!("unwind", wasm_unwind);
run_test!("utf8-custom-section-id", wasm_utf8_custom_section_id);
run_test!("utf8-import-field", wasm_utf8_import_field);
run_test!("utf8-import-module", wasm_utf8_import_module);
run_test!("utf8-invalid-encoding", wasm_utf8_invalid_encoding);
