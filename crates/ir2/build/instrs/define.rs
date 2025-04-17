use super::*;
use std::format;

pub fn define_instrs(ctx: &mut Context) {
    define_copy_instrs(ctx);
    define_return_instrs(ctx);
    define_global_instrs(ctx);
    define_br_table_instrs(ctx);
    define_fused_cmp_branch_instrs_commutative(ctx);
    define_fused_cmp_branch_instrs(ctx);
    define_iunop_instrs(ctx);
    define_funop_instrs(ctx);
    define_conversion_instrs(ctx);
    define_ibinop_instrs_commutative(ctx);
    define_fbinop_instrs_commutative(ctx);
    define_ibinop_instrs(ctx);
    define_fbinop_instrs(ctx);
    define_load_instrs(ctx);
    define_store_instrs(ctx);
    define_select_instrs(ctx);
    define_table_instrs(ctx);
    define_memory_instrs(ctx);
    define_call_instrs(ctx);
}

fn define_br_table_instrs(ctx: &mut Context) {
    for index in [Operand::Reg, Operand::Stack] {
        let index_id = index.id();
        let index_ty = index.ty(ValTy::I32);
        ctx.push_instr(op!(
            name: format!("BranchTable0_{index_id}"),
            fields: [
                index: index_ty,
                len_targets: ImmediateTy::U32,
            ],
        ));
        ctx.push_instr(op!(
            name: format!("BranchTable_{index_id}"),
            fields: [
                index: index_ty,
                len_targets: ImmediateTy::U32,
            ],
        ));
    }
}

fn define_fused_cmp_branch_instrs_impl(
    ctx: &mut Context,
    commutative: bool,
    ops_and_tys: impl IntoIterator<Item = (&'static str, ValTy)>,
) {
    let lhs_in = [Operand::Reg, Operand::Stack, Operand::Immediate];
    let rhs_in = [Operand::Reg, Operand::Stack, Operand::Immediate];
    for (op, ty) in ops_and_tys {
        let op = format!("{ty}{op}");
        for lhs in &lhs_in {
            for rhs in &rhs_in {
                if lhs.is_imm() && rhs.is_imm() {
                    continue;
                }
                if lhs.is_reg() && rhs.is_reg() {
                    continue;
                }
                if commutative && lhs > rhs {
                    continue;
                }
                let lhs_id = lhs.id();
                let rhs_id = rhs.id();
                ctx.push_instr(op! {
                    name: format!("{op}Branch_{lhs_id}{rhs_id}"),
                    fields: [
                        lhs: lhs.ty(ty),
                        rhs: rhs.ty(ty),
                        offset: ImmediateTy::BranchOffset,
                    ],
                });
            }
        }
    }
}

fn define_fused_cmp_branch_instrs_commutative(ctx: &mut Context) {
    define_fused_cmp_branch_instrs_impl(
        ctx,
        true,
        [
            ("Eq", ValTy::I32),
            ("Eq", ValTy::I64),
            ("Eq", ValTy::F32),
            ("Eq", ValTy::F64),
            ("Ne", ValTy::I32),
            ("Ne", ValTy::I64),
            ("Ne", ValTy::F32),
            ("Ne", ValTy::F64),
        ],
    )
}

fn define_fused_cmp_branch_instrs(ctx: &mut Context) {
    define_fused_cmp_branch_instrs_impl(
        ctx,
        false,
        [
            ("LtS", ValTy::I32),
            ("LtS", ValTy::I64),
            ("LtU", ValTy::I32),
            ("LtU", ValTy::I64),
            ("LeS", ValTy::I32),
            ("LeS", ValTy::I64),
            ("LeU", ValTy::I32),
            ("LeU", ValTy::I64),
            ("Lt", ValTy::F32),
            ("Lt", ValTy::F64),
            ("Le", ValTy::F32),
            ("Le", ValTy::F64),
        ],
    )
}

fn define_unop_instrs(
    ctx: &mut Context,
    ops: impl IntoIterator<Item = &'static str>,
    tys: impl IntoIterator<Item = ValTy> + Clone,
) {
    for op in ops {
        for ty in tys.clone() {
            let op = format!("{ty}{op}");
            for result in [Operand::Reg, Operand::Stack] {
                for input in [Operand::Reg, Operand::Stack] {
                    let result_id = result.id();
                    let input_id = input.id();
                    ctx.push_instr(op! {
                        name: format!("{op}_{result_id}{input_id}"),
                        fields: [
                            result: result.ty(ty),
                            input: input.ty(ty),
                        ],
                    });
                }
            }
        }
    }
}

fn define_iunop_instrs(ctx: &mut Context) {
    define_unop_instrs(ctx, ["Popcnt", "Clz", "Ctz"], [ValTy::I32, ValTy::I64])
}

fn define_funop_instrs(ctx: &mut Context) {
    define_unop_instrs(
        ctx,
        ["Abs", "Neg", "Ceil", "Floor", "Trunc", "Nearest", "Sqrt"],
        [ValTy::F32, ValTy::F64],
    )
}

fn define_conversion_instrs(ctx: &mut Context) {
    let ops = [
        ("Demote", ValTy::F32, ValTy::F64),
        ("Promote", ValTy::F64, ValTy::F32),
        ("ConvertI32S", ValTy::F32, ValTy::I32),
        ("ConvertI32U", ValTy::F32, ValTy::I32),
        ("ConvertI64S", ValTy::F32, ValTy::I64),
        ("ConvertI64U", ValTy::F32, ValTy::I64),
        ("ConvertI32S", ValTy::F64, ValTy::I32),
        ("ConvertI32U", ValTy::F64, ValTy::I32),
        ("ConvertI64S", ValTy::F64, ValTy::I64),
        ("ConvertI64U", ValTy::F64, ValTy::I64),
        ("TruncF32S", ValTy::I32, ValTy::F32),
        ("TruncF32U", ValTy::I32, ValTy::F32),
        ("TruncF64S", ValTy::I32, ValTy::F64),
        ("TruncF64U", ValTy::I32, ValTy::F64),
        ("TruncF32S", ValTy::I64, ValTy::F32),
        ("TruncF32U", ValTy::I64, ValTy::F32),
        ("TruncF64S", ValTy::I64, ValTy::F64),
        ("TruncF64U", ValTy::I64, ValTy::F64),
        ("TruncSatF32S", ValTy::I32, ValTy::F32),
        ("TruncSatF32U", ValTy::I32, ValTy::F32),
        ("TruncSatF64S", ValTy::I32, ValTy::F64),
        ("TruncSatF64U", ValTy::I32, ValTy::F64),
        ("TruncSatF32S", ValTy::I64, ValTy::F32),
        ("TruncSatF32U", ValTy::I64, ValTy::F32),
        ("TruncSatF64S", ValTy::I64, ValTy::F64),
        ("TruncSatF64U", ValTy::I64, ValTy::F64),
        ("Extend8S", ValTy::I32, ValTy::I32),
        ("Extend16S", ValTy::I32, ValTy::I32),
        ("Extend8S", ValTy::I64, ValTy::I64),
        ("Extend16S", ValTy::I64, ValTy::I64),
        ("Extend32S", ValTy::I64, ValTy::I64),
        ("WrapI64", ValTy::I32, ValTy::I64),
    ];
    for (op, result_ty, input_ty) in ops {
        let op = format!("{result_ty}{op}");
        for result in [Operand::Reg, Operand::Stack] {
            for input in [Operand::Reg, Operand::Stack] {
                let result_id = result.id();
                let input_id = input.id();
                ctx.push_instr(op! {
                    name: format!("{op}_{result_id}{input_id}"),
                    fields: [
                        result: result.ty(result_ty),
                        input: input.ty(input_ty),
                    ],
                });
            }
        }
    }
}

fn define_load_instrs(ctx: &mut Context) {
    let ops_and_tys = [
        ("Load", ValTy::I32),
        ("Load", ValTy::I64),
        ("Load", ValTy::F32),
        ("Load", ValTy::F64),
        ("Load8S", ValTy::I32),
        ("Load8S", ValTy::I64),
        ("Load8U", ValTy::I32),
        ("Load8U", ValTy::I64),
        ("Load16S", ValTy::I32),
        ("Load16S", ValTy::I64),
        ("Load16U", ValTy::I32),
        ("Load16U", ValTy::I64),
        ("Load32S", ValTy::I64),
        ("Load32U", ValTy::I64),
    ];
    let results = [Operand::Reg, Operand::Stack];
    let ptrs = [Operand::Reg, Operand::Stack, Operand::Immediate];
    for (op, ty) in ops_and_tys {
        for mem0 in [false, true] {
            let mem0_str: &str = match mem0 {
                true => "Mem0",
                false => "",
            };
            let op = format!("{ty}{op}{mem0_str}");
            for result in results {
                if !mem0 && result.is_stack() {
                    continue;
                }
                for ptr in ptrs {
                    let result_id = result.id();
                    let ptr_id = ptr.id();
                    let instr = match mem0 {
                        true => op! {
                            name: format!("{op}_{result_id}{ptr_id}"),
                            fields: [
                                result: result.ty(ty),
                                ptr: ptr.ty(ValTy::I64),
                                offset: ImmediateTy::Offset,
                            ],
                        },
                        false => op! {
                            name: format!("{op}_{result_id}{ptr_id}"),
                            fields: [
                                result: result.ty(ty),
                                ptr: ptr.ty(ValTy::I64),
                                offset: ImmediateTy::Offset,
                                memory: ImmediateTy::Memory,
                            ],
                        },
                    };
                    ctx.push_instr(instr);
                }
            }
        }
    }
}

fn define_store_instrs(ctx: &mut Context) {
    let ops_and_tys = [
        ("Store", ValTy::I32),
        ("Store", ValTy::I64),
        ("Store", ValTy::F32),
        ("Store", ValTy::F64),
        ("Store8", ValTy::I32),
        ("Store8", ValTy::I64),
        ("Store16", ValTy::I32),
        ("Store16", ValTy::I64),
        ("Store32", ValTy::I64),
    ];
    let ptrs = [Operand::Reg, Operand::Stack, Operand::Immediate];
    let values = [Operand::Reg, Operand::Stack, Operand::Immediate];
    for (op, ty) in ops_and_tys {
        for mem0 in [false, true] {
            let mem0_id = match mem0 {
                true => "Mem0",
                false => "",
            };
            let op = format!("{ty}{op}{mem0_id}");
            for ptr in &ptrs {
                for value in &values {
                    if matches!(ty, ValTy::I32 | ValTy::I64) && ptr.is_reg() && value.is_reg() {
                        continue;
                    }
                    let ptr_id = ptr.id();
                    let value_id = value.id();
                    let instr = match (ptr, mem0) {
                        (Operand::Immediate, true) => {
                            op! {
                                name: format!("{op}_{ptr_id}{value_id}"),
                                fields: [
                                    address: ImmediateTy::Address,
                                    value: value.ty(ty),
                                ],
                            }
                        }
                        (Operand::Immediate, false) => {
                            op! {
                                name: format!("{op}_{ptr_id}{value_id}"),
                                fields: [
                                    address: ImmediateTy::Address,
                                    value: value.ty(ty),
                                    memory: ImmediateTy::Memory,
                                ],
                            }
                        }
                        (_, true) => {
                            op! {
                                name: format!("{op}_{ptr_id}{value_id}"),
                                fields: [
                                    ptr: ptr.ty(ValTy::I64),
                                    value: value.ty(ty),
                                    offset: ImmediateTy::Offset,
                                ],
                            }
                        }
                        (_, false) => {
                            op! {
                                name: format!("{op}_{ptr_id}{value_id}"),
                                fields: [
                                    ptr: ptr.ty(ValTy::I64),
                                    value: value.ty(ty),
                                    offset: ImmediateTy::Offset,
                                    memory: ImmediateTy::Memory,
                                ],
                            }
                        }
                    };
                    ctx.push_instr(instr);
                }
            }
        }
    }
}

fn define_binop_instrs(
    ctx: &mut Context,
    commutative: bool,
    ops: impl IntoIterator<Item = &'static str>,
    tys: impl IntoIterator<Item = ValTy> + Clone,
) {
    let results = match commutative {
        true => &[Operand::Reg, Operand::Stack][..],
        false => &[Operand::Reg][..],
    };
    let lhs_in = [Operand::Reg, Operand::Stack, Operand::Immediate];
    let rhs_in = [Operand::Reg, Operand::Stack, Operand::Immediate];
    for op in ops {
        for ty in tys.clone() {
            let op = format!("{ty}{op}");
            for result in results {
                for lhs in lhs_in {
                    for rhs in rhs_in {
                        if lhs.is_reg() && rhs.is_reg() {
                            continue;
                        }
                        if lhs.is_imm() && rhs.is_imm() {
                            continue;
                        }
                        if commutative && lhs > rhs {
                            continue;
                        }
                        let result_id = result.id();
                        let lhs_id = lhs.id();
                        let rhs_id = rhs.id();
                        ctx.push_instr(op! {
                            name: format!("{op}_{result_id}{lhs_id}{rhs_id}"),
                            fields: [
                                result: result.ty(ty),
                                lhs: lhs.ty(ty),
                                rhs: rhs.ty(ty),
                            ],
                        });
                    }
                }
            }
        }
    }
}

fn define_ibinop_instrs_commutative(ctx: &mut Context) {
    define_binop_instrs(
        ctx,
        true,
        ["Add", "Mul", "And", "Or", "Xor", "Eq", "Ne"],
        [ValTy::I32, ValTy::I64],
    )
}

fn define_fbinop_instrs_commutative(ctx: &mut Context) {
    define_binop_instrs(
        ctx,
        true,
        ["Add", "Mul", "Eq", "Ne", "Min", "Max"],
        [ValTy::F32, ValTy::F64],
    )
}

fn define_ibinop_instrs(ctx: &mut Context) {
    define_binop_instrs(
        ctx,
        false,
        [
            "Sub", "LtS", "LtU", "LeS", "LeU", "DivS", "DivU", "RemS", "RemU", "Shl", "ShrS",
            "ShrU", "Rotl", "Rotr",
        ],
        [ValTy::I32, ValTy::I64],
    )
}

fn define_fbinop_instrs(ctx: &mut Context) {
    define_binop_instrs(
        ctx,
        false,
        ["Sub", "Div", "Copysign"],
        [ValTy::F32, ValTy::F64],
    )
}

fn define_copy_instrs(ctx: &mut Context) {
    let stack_id = Operand::Stack.id();
    ctx.push_instr(op! {
        name: format!("Copy1_{stack_id}"),
        fields: [
            result: FieldTy::Stack,
            value: FieldTy::Stack,
        ],
    });
    ctx.push_instr(op! {
        name: "Copy",
        fields: [
            result: FieldTy::Stack,
            len_values: ImmediateTy::Usize,
        ],
    });
    for ty in [ValTy::I32, ValTy::I64, ValTy::F32, ValTy::F64] {
        for value in [Operand::Reg, Operand::Immediate] {
            if matches!(ty, ValTy::I32) && value.is_reg() {
                continue;
            }
            let op = format!("Copy1{ty}");
            let value_id = value.id();
            ctx.push_instr(op! {
                name: format!("{op}_{value_id}"),
                fields: [
                    result: FieldTy::Stack,
                    value: value.ty(ty),
                ],
            });
        }
    }
}

fn define_global_instrs(ctx: &mut Context) {
    define_global_get_instrs(ctx);
    define_global_set_instrs(ctx);
}

fn define_global_get_instrs(ctx: &mut Context) {
    let stack_id = Operand::Stack.id();
    ctx.push_instr(op! {
        name: format!("GlobalGet_{stack_id}"),
        fields: [
            result: FieldTy::Stack,
            global: ImmediateTy::Global,
        ],
    });
    for ty in [ValTy::I32, ValTy::I64, ValTy::F32, ValTy::F64] {
        let result_id = Operand::Reg.id();
        ctx.push_instr(op! {
            name: format!("GlobalGet{ty}_{result_id}"),
            fields: [
                result: Operand::Reg.ty(ty),
                global: ImmediateTy::Global,
            ],
        });
    }
}

fn define_global_set_instrs(ctx: &mut Context) {
    let stack_id = Operand::Stack.id();
    ctx.push_instr(op! {
        name: format!("GlobalSet_{stack_id}"),
        fields: [
            global: ImmediateTy::Global,
            value: FieldTy::Stack,
        ],
    });
    for value in [Operand::Reg, Operand::Immediate] {
        for ty in [ValTy::I32, ValTy::I64, ValTy::F32, ValTy::F64] {
            let value_id = value.id();
            ctx.push_instr(op! {
                name: format!("GlobalSet{ty}_{value_id}"),
                fields: [
                    global: ImmediateTy::Global,
                    value: value.ty(ty),
                ],
            });
        }
    }
}

fn define_return_instrs(ctx: &mut Context) {
    // Return0
    ctx.push_instr(op! {
        name: "Return0",
        fields: [],
    });
    // Return1
    {
        let stack_id = Operand::Stack.id();
        ctx.push_instr(op! {
            name: format!("Return1_{stack_id}"),
            fields: [
                value: FieldTy::Stack,
            ],
        });
    }
    // Return (many)
    ctx.push_instr(op! {
        name: "Return",
        fields: [
            len_values: ImmediateTy::Usize,
        ],
    });
    // Return1 (reg)
    for value in [Operand::Reg, Operand::Immediate] {
        for ty in [ValTy::I32, ValTy::I64, ValTy::F32, ValTy::F64] {
            let value_id = value.id();
            ctx.push_instr(op! {
                name: format!("Return1{ty}_{value_id}"),
                fields: [
                    value: value.ty(ty),
                ],
            });
        }
    }
}

fn define_select_instrs(ctx: &mut Context) {
    // Select without type:
    ctx.push_instr(op! {
        name: "Select",
        fields: [
            result: FieldTy::Reg,
            condition: FieldTy::Stack,
            lhs: FieldTy::Stack,
            rhs: FieldTy::Stack,
        ],
    });
    // Select with type:
    for ty in [ValTy::I32, ValTy::I64, ValTy::F32, ValTy::F64] {
        for result in [Operand::Reg] {
            let result_id = result.id();
            let result_ty = result.ty(ty);
            for condition in [Operand::Reg, Operand::Stack] {
                if matches!(ty, ValTy::I32) && condition.is_reg() {
                    continue;
                }
                let condition_id = condition.id();
                let condition_ty = condition.ty(ty);
                for lhs in [Operand::Reg, Operand::Stack, Operand::Immediate] {
                    if matches!(ty, ValTy::I32) && lhs.is_reg() {
                        continue;
                    }
                    let lhs_id = lhs.id();
                    let lhs_ty = lhs.ty(ty);
                    for rhs in [Operand::Reg, Operand::Stack, Operand::Immediate] {
                        if matches!(ty, ValTy::I32) && rhs.is_reg() {
                            continue;
                        }
                        if result.is_stack()
                            && condition.is_stack()
                            && lhs.is_stack()
                            && rhs.is_stack()
                        {
                            continue;
                        }
                        if u8::from(condition.is_reg())
                            + u8::from(lhs.is_reg())
                            + u8::from(rhs.is_reg())
                            >= 2
                        {
                            continue;
                        }
                        let rhs_id = rhs.id();
                        let rhs_ty = rhs.ty(ty);
                        ctx.push_instr(op! {
                            name: format!("Select{ty}_{result_id}{condition_id}{lhs_id}{rhs_id}"),
                            fields: [
                                result: result_ty,
                                condition: condition_ty,
                                lhs: lhs_ty,
                                rhs: rhs_ty,
                            ],
                        });
                    }
                }
            }
        }
    }
}

fn define_table_instrs(ctx: &mut Context) {
    define_table_size_instrs(ctx);
    define_table_get_instrs(ctx);
    define_table_set_instrs(ctx);
    define_table_grow_instrs(ctx);
    define_table_copy_instrs(ctx);
    define_table_fill_instrs(ctx);
    define_table_init_instrs(ctx);
}

fn define_table_size_instrs(ctx: &mut Context) {
    for result in [Operand::Reg, Operand::Stack] {
        let result_id = result.id();
        ctx.push_instr(op! {
            name: format!("TableSize_{result_id}"),
            fields: [
                result: result.ty(ValTy::I64),
                table: ImmediateTy::Table,
            ],
        });
    }
}

fn define_table_get_instrs(ctx: &mut Context) {
    let result_id = Operand::Reg.id();
    let result_ty = Operand::Reg.ty(ValTy::I64);
    for index in [Operand::Reg, Operand::Stack, Operand::Immediate] {
        let index_id = index.id();
        ctx.push_instr(op! {
            name: format!("TableGet_{result_id}{index_id}"),
            fields: [
                result: result_ty,
                index: index.ty(ValTy::I64),
                table: ImmediateTy::Table,
            ],
        });
    }
}

fn define_table_set_instrs(ctx: &mut Context) {
    for index in [Operand::Reg, Operand::Stack, Operand::Immediate] {
        let index_id = index.id();
        let index_ty = index.ty(ValTy::I64);
        for value in [Operand::Reg, Operand::Stack, Operand::Immediate] {
            if index.is_reg() && value.is_reg() {
                continue;
            }
            let value_id = value.id();
            let value_ty = value.ty(ValTy::I32);
            ctx.push_instr(op! {
                name: format!("TableSet_{index_id}{value_id}"),
                fields: [
                    index: index_ty,
                    value: value_ty,
                    table: ImmediateTy::Table,
                ],
            });
        }
    }
}

fn define_table_grow_instrs(ctx: &mut Context) {
    ctx.push_instr(op! {
        name: "TableGrow",
        fields: [
            result: FieldTy::Stack,
            delta: FieldTy::Stack,
            table: ImmediateTy::Table,
        ],
    });
}

fn define_table_copy_instrs(ctx: &mut Context) {
    ctx.push_instr(op! {
        name: "TableCopy",
        fields: [
            dst_index: FieldTy::Stack,
            src_index: FieldTy::Stack,
            len: FieldTy::Stack,
            dst_table: ImmediateTy::Table,
            src_table: ImmediateTy::Table,
        ],
    });
}

fn define_table_fill_instrs(ctx: &mut Context) {
    ctx.push_instr(op! {
        name: "TableFill",
        fields: [
            dst_index: FieldTy::Stack,
            value: FieldTy::Stack,
            len: FieldTy::Stack,
            table: ImmediateTy::Table,
        ],
    });
}

fn define_table_init_instrs(ctx: &mut Context) {
    ctx.push_instr(op! {
        name: "TableInit",
        fields: [
            dst_index: FieldTy::Stack,
            src_index: FieldTy::Stack,
            len: FieldTy::Stack,
            table: ImmediateTy::Table,
            elem: ImmediateTy::Elem,
        ],
    });
}

fn define_memory_instrs(ctx: &mut Context) {
    define_memory_size_instrs(ctx);
    define_memory_grow_instrs(ctx);
    define_memory_copy_instrs(ctx);
    define_memory_fill_instrs(ctx);
    define_memory_init_instrs(ctx);
}

fn define_memory_size_instrs(ctx: &mut Context) {
    for result in [Operand::Reg, Operand::Stack] {
        let result_id = result.id();
        ctx.push_instr(op! {
            name: format!("MemorySize_{result_id}"),
            fields: [
                result: result.ty(ValTy::I64),
                memory: ImmediateTy::Memory,
            ],
        });
    }
}

fn define_memory_grow_instrs(ctx: &mut Context) {
    ctx.push_instr(op! {
        name: "MemoryGrow",
        fields: [
            result: FieldTy::Reg,
            delta: FieldTy::Stack,
            memory: ImmediateTy::Memory,
        ],
    })
}

fn define_memory_copy_instrs(ctx: &mut Context) {
    ctx.push_instr(op! {
        name: "MemoryCopy",
        fields: [
            dst_index: FieldTy::Stack,
            src_index: FieldTy::Stack,
            len: FieldTy::Stack,
            dst_memory: ImmediateTy::Memory,
            src_memory: ImmediateTy::Memory,
        ],
    })
}

fn define_memory_fill_instrs(ctx: &mut Context) {
    ctx.push_instr(op! {
        name: "MemoryFill",
        fields: [
            dst_index: FieldTy::Stack,
            value: FieldTy::Stack,
            len: FieldTy::Stack,
            memory: ImmediateTy::Memory,
        ],
    })
}

fn define_memory_init_instrs(ctx: &mut Context) {
    ctx.push_instr(op! {
        name: "MemoryInit",
        fields: [
            dst_index: FieldTy::Stack,
            src_index: FieldTy::Stack,
            len: FieldTy::Stack,
            memory: ImmediateTy::Memory,
            data: ImmediateTy::Data,
        ],
    })
}

fn define_call_instrs(ctx: &mut Context) {
    define_call_internal_instrs(ctx);
    define_call_imported_instrs(ctx);
    define_call_indirect_instrs(ctx);
}

fn define_call_internal_instrs(ctx: &mut Context) {
    for op in ["Call", "ReturnCall"] {
        ctx.push_instr(op! {
            name: format!("{op}Internal"),
            fields: [
                func: ImmediateTy::WasmFunc,
                len_params: ImmediateTy::Usize,
                len_results: ImmediateTy::Usize,
            ],
        })
    }
}

fn define_call_imported_instrs(ctx: &mut Context) {
    for op in ["Call", "ReturnCall"] {
        ctx.push_instr(op! {
            name: format!("{op}Imported"),
            fields: [
                func: ImmediateTy::Func,
                len_params: ImmediateTy::Usize,
                len_results: ImmediateTy::Usize,
            ],
        })
    }
}

fn define_call_indirect_instrs(ctx: &mut Context) {
    for op in ["Call", "ReturnCall"] {
        for index in [Operand::Reg, Operand::Stack, Operand::Immediate] {
            let index_id = index.id();
            ctx.push_instr(op! {
                name: format!("{op}Indirect_{index_id}"),
                fields: [
                    table: ImmediateTy::Table,
                    index: index.ty(ValTy::I64),
                    len_params: ImmediateTy::Usize,
                    len_results: ImmediateTy::Usize,
                ],
            })
        }
    }
}
