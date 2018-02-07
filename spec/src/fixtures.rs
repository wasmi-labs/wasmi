macro_rules! run_test {
    ($label: expr, $test_name: ident) => (
        #[test]
        fn $test_name() {
            ::run::spec($label)
        }
    );
}

run_test!("address-offset-range.fail", wasm_address_offset_range_fail);
run_test!("address", wasm_address);
run_test!("binary", wasm_binary);
run_test!("block-end-label-mismatch.fail", wasm_block_end_label_mismatch_fail);
run_test!("block-end-label-superfluous.fail", wasm_block_end_label_superfluous_fail);
run_test!("block", wasm_block);
run_test!("br_if", wasm_br_if);
run_test!("br_table", wasm_br_table);
run_test!("br", wasm_br);
run_test!("break-drop", wasm_break_drop);
run_test!("call_indirect", wasm_call_indirect);
run_test!("call", wasm_call);
run_test!("comments", wasm_comments);
// TODO: commented out until sNaN issue is resolved:
// https://github.com/NikVolf/parity-wasm/blob/b5aaf103cf28f1e36df832f4883f55043e67894b/src/interpreter/value.rs#L510
// run_test!("conversions", wasm_conversions);
run_test!("custom_section", wasm_custom_section);
run_test!("endianness", wasm_endianness);
run_test!("f32_bitwise", wasm_f32_bitwise);
run_test!("f32_cmp", wasm_f32_cmp);
run_test!("f32.load32.fail", wasm_f32_load32_fail);
run_test!("f32.load64.fail", wasm_f32_load64_fail);
run_test!("f32.store32.fail", wasm_f32_store32_fail);
run_test!("f32.store64.fail", wasm_f32_store64_fail);
run_test!("f32", wasm_f32);
run_test!("f64_bitwise", wasm_f64_bitwise);
run_test!("f64_cmp", wasm_f64_cmp);
run_test!("f64.load32.fail", wasm_f64_load32_fail);
run_test!("f64.load64.fail", wasm_f64_load64_fail);
run_test!("f64.store32.fail", wasm_f64_store32_fail);
run_test!("f64.store64.fail", wasm_f64_store64_fail);
run_test!("f64", wasm_f64);
run_test!("fac", wasm_fac);
// TODO: commented out until sNaN issue is resolved:
// https://github.com/NikVolf/parity-wasm/blob/b5aaf103cf28f1e36df832f4883f55043e67894b/src/interpreter/value.rs#L510
// run_test!("float_exprs", wasm_float_exprs);
// run_test!("float_literals", wasm_float_literals);
// run_test!("float_memory", wasm_float_memory);
run_test!("float_misc", wasm_float_misc);
run_test!("forward", wasm_forward);
run_test!("func_ptrs", wasm_func_ptrs);
run_test!("func-local-after-body.fail", wasm_func_local_after_body_fail);
run_test!("func-local-before-param.fail", wasm_func_local_before_param_fail);
run_test!("func-local-before-result.fail", wasm_func_local_before_result_fail);
run_test!("func-param-after-body.fail", wasm_func_param_after_body_fail);
run_test!("func-result-after-body.fail", wasm_func_result_after_body_fail);
run_test!("func-result-before-param.fail", wasm_func_result_before_param_fail);
run_test!("func", wasm_func);
run_test!("get_local", wasm_get_local);
run_test!("globals", wasm_globals);
run_test!("i32.load32_s.fail", wasm_i32_load32s_fail);
run_test!("i32.load32_u.fail", wasm_i32_load32u_fail);
run_test!("i32.load64_s.fail", wasm_i32_load64s_fail);
run_test!("i32.load64_u.fail", wasm_i32_load64u_fail);
run_test!("i32.store32.fail", wasm_i32_store32_fail);
run_test!("i32.store64.fail", wasm_i32_store64_fail);
run_test!("i32", wasm_i32);
run_test!("i64.load64_s.fail", wasm_i64_load64s_fail);
run_test!("i64.load64_u.fail", wasm_i64_load64u_fail);
run_test!("i64.store64.fail", wasm_i64_store64_fail);
run_test!("i64", wasm_i64);
run_test!("if-else-end-label-mismatch.fail", wasm_if_else_end_label_mismatch_fail);
run_test!("if-else-end-label-superfluous.fail", wasm_if_else_end_label_superfluous_fail);
run_test!("if-else-label-mismatch.fail", wasm_if_else_label_mismatch_fail);
run_test!("if-else-label-superfluous.fail", wasm_if_else_label_superfluous_fail);
run_test!("if-end-label-mismatch.fail", wasm_if_end_label_mismatch_fail);
run_test!("if-end-label-superfluous.fail", wasm_if_end_label_superfluous_fail);
run_test!("if", wasm_if);
run_test!("import-after-func.fail", wasm_import_after_func_fail);
run_test!("import-after-global.fail", wasm_import_after_global_fail);
run_test!("import-after-memory.fail", wasm_import_after_memory_fail);
run_test!("import-after-table.fail", wasm_import_after_table_fail);
run_test!("imports", wasm_imports);
run_test!("int_exprs", wasm_int_exprs);
run_test!("int_literals", wasm_int_literals);
run_test!("labels", wasm_labels);
run_test!("left-to-right", wasm_left_to_right);
run_test!("linking", wasm_linking);
run_test!("load-align-0.fail", wasm_load_align_0_fail);
run_test!("load-align-big.fail", wasm_load_align_big_fail);
run_test!("load-align-odd.fail", wasm_load_align_odd_fail);
run_test!("loop-end-label-mismatch.fail", wasm_end_label_mismatch_fail);
run_test!("loop-end-label-superfluous.fail", wasm_end_label_superfluous_fail);
run_test!("loop", wasm_loop);
run_test!("memory_redundancy", wasm_memory_redundancy);
run_test!("memory_trap", wasm_memory_trap);
run_test!("memory", wasm_memory);
run_test!("names", wasm_names);
run_test!("nop", wasm_nop);
run_test!("of_string-overflow-hex-u32.fail", wasm_of_string_overflow_hex_u32_fail);
run_test!("of_string-overflow-hex-u64.fail", wasm_of_string_overflow_hex_u64_fail);
run_test!("of_string-overflow-s32.fail", wasm_of_string_overflow_s32_fail);
run_test!("of_string-overflow-s64.fail", wasm_of_string_overflow_s64_fail);
run_test!("of_string-overflow-u32.fail", wasm_of_string_overflow_u32_fail);
run_test!("of_string-overflow-u64.fail", wasm_of_string_overflow_u64_fail);
run_test!("resizing", wasm_resizing);
run_test!("return", wasm_return);
run_test!("select", wasm_select);
run_test!("set_local", wasm_set_local);
run_test!("skip-stack-guard-page", wasm_skip_stack_guard_page);
run_test!("stack", wasm_stack);
run_test!("start", wasm_start);
run_test!("store_retval", wasm_store_retval);
run_test!("store-align-0.fail", wasm_store_align_0_fail);
run_test!("store-align-big.fail", wasm_store_align_big_fail);
run_test!("store-align-odd.fail", wasm_store_align_odd_fail);
run_test!("switch", wasm_switch);
run_test!("tee_local", wasm_tee_local);
run_test!("traps", wasm_traps);
run_test!("typecheck", wasm_typecheck);
run_test!("unreachable", wasm_unreachable);
run_test!("unreached-invalid", wasm_unreached_invalid);
run_test!("unwind", wasm_unwind);
run_test!("utf8-custom-section-id", wasm_utf8_custom_section_id);
run_test!("utf8-import-field", wasm_utf8_import_field);
run_test!("utf8-import-module", wasm_utf8_import_module);
run_test!("utf8-invalid-encoding", wasm_utf8_invalid_encoding);
